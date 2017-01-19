package main

import (
	"log"
	"net"
	"sync"
)

type client struct {
	conn  *net.TCPConn
	wchan chan []byte
}

func newClient(conn *net.TCPConn) *client {
	return &client{
		conn:  conn,
		wchan: make(chan []byte),
	}
}

func (cli *client) write(data []byte) {
	cli.wchan <- data
}

func (cli *client) close() {
	close(cli.wchan)
	cli.conn.Close()
}

func (cli *client) readLoop(rchan chan []byte, rmChan chan *client) {
	defer cli.conn.Close()
	defer func() { rmChan <- cli }()
	buff := make([]byte, 65536)
	for {
		n, err := cli.conn.Read(buff)
		if n > 0 {
			msg := make([]byte, n)
			copy(msg, buff[:n])
			rchan <- msg
		}
		if err != nil {
			log.Println(err)
			return
		} else if n <= 0 {
			return
		}
	}
}

func (cli *client) writeLoop() {
	for {
		buff, ok := <-cli.wchan
		if !ok {
			return
		}
		cli.conn.Write(buff)
	}
}

func (cli *client) id() string {
	return cli.conn.RemoteAddr().String()
}

type room struct {
	cls    []*client
	rChan  chan []byte
	coChan chan *net.TCPConn
	rmChan chan *client
	lock   *sync.RWMutex
}

func newRoom() *room {
	return &room{
		cls:    make([]*client, 0),
		rChan:  make(chan []byte),
		coChan: make(chan *net.TCPConn),
		rmChan: make(chan *client),
		lock:   new(sync.RWMutex),
	}
}

func (r *room) coLoop() {
	for co := range r.coChan {
		cli := newClient(co)
		go cli.writeLoop()
		r.lock.Lock()
		r.cls = append(r.cls, cli)
		r.lock.Unlock()
		go cli.readLoop(r.rChan, r.rmChan)
		log.Printf("New client connected, now %d clients", len(r.cls))
	}
}

func (r *room) broadcastLoop() {
	for msg := range r.rChan {
		r.lock.RLock()
		for _, cli := range r.cls {
			cli.write(msg)
		}
		r.lock.RUnlock()
	}
}

func (r *room) rmLoop() {
	for cli := range r.rmChan {
		log.Println("Removing a client")
		r.lock.Lock()
		for i, c := range r.cls {
			if c.id() == cli.id() {
				r.cls = append(r.cls[:i], r.cls[i+1:]...)
				break
			}
		}
		r.lock.Unlock()
		cli.close()
		log.Printf("Client removed, %d client remaining", len(r.cls))
	}
}

func createTCPServer(coChan chan *net.TCPConn) error {
	tcpAddr, err := net.ResolveTCPAddr("tcp", ":4242")
	if err != nil {
		return err
	}
	ln, err := net.ListenTCP("tcp", tcpAddr)
	if err != nil {
		return err
	}
	defer close(coChan)
	defer ln.Close()
	for {
		conn, err := ln.AcceptTCP()
		if err != nil {
			return err
		}
		coChan <- conn
	}
}

func main() {
	r := newRoom()
	go r.coLoop()
	go r.broadcastLoop()
	go r.rmLoop()
	createTCPServer(r.coChan)
}
