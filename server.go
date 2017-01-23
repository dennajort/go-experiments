package main

import (
	"log"
	"net"
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

func (cli *client) readLoop(r *room) {
	defer cli.conn.Close()
	defer r.removeClient(cli)
	buff := make([]byte, 65536)
	for {
		n, err := cli.conn.Read(buff)
		if n > 0 {
			msg := make([]byte, n)
			copy(msg, buff[:n])
			r.broadcast(msg)
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
	cls     []*client
	cmdChan chan func()
}

func newRoom() *room {
	return &room{
		cls:     make([]*client, 0),
		cmdChan: make(chan func()),
	}
}

func (r *room) broadcast(msg []byte) {
	r.cmdChan <- func() {
		for _, cli := range r.cls {
			cli.write(msg)
		}
	}
}

func (r *room) addClient(co *net.TCPConn) {
	cli := newClient(co)
	go cli.writeLoop()
	r.cmdChan <- func() {
		r.cls = append(r.cls, cli)
		go cli.readLoop(r)
		log.Printf("New client connected, now %d clients", len(r.cls))
	}
}

func (r *room) removeClient(cli *client) {
	r.cmdChan <- func() {
		log.Println("Removing a client")
		for i, c := range r.cls {
			if c.id() == cli.id() {
				r.cls = append(r.cls[:i], r.cls[i+1:]...)
				break
			}
		}
		cli.close()
		log.Printf("Client removed, %d client remaining", len(r.cls))
	}
}

func (r *room) cmdLoop() {
	for cmd := range r.cmdChan {
		cmd()
	}
}

func (r *room) listen() error {
	tcpAddr, err := net.ResolveTCPAddr("tcp", ":4242")
	if err != nil {
		return err
	}
	ln, err := net.ListenTCP("tcp", tcpAddr)
	if err != nil {
		return err
	}
	defer ln.Close()
	for {
		co, err := ln.AcceptTCP()
		if err != nil {
			return err
		}
		r.addClient(co)
	}
}

func main() {
	r := newRoom()
	go r.cmdLoop()
	r.listen()
}
