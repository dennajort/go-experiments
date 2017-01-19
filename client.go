package main

import (
	"io"
	"log"
	"net"
	"os"
	"runtime"
	"sync"
)

func main() {
	runtime.GOMAXPROCS(runtime.NumCPU())
	raddr, err := net.ResolveTCPAddr("tcp", "127.0.0.1:4242")
	if err != nil {
		log.Fatalln(err)
	}
	conn, err := net.DialTCP("tcp", nil, raddr)
	if err != nil {
		log.Fatalln(err)
	}
	wg := new(sync.WaitGroup)
	wg.Add(2)
	go func() {
		defer wg.Done()
		io.CopyBuffer(conn, os.Stdin, make([]byte, 65536))
		conn.CloseWrite()
	}()
	go func() {
		defer wg.Done()
		io.CopyBuffer(os.Stdout, conn, make([]byte, 65536))
		conn.CloseRead()
	}()
	wg.Wait()
}
