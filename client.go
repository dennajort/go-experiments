package main

import (
	"io"
	"log"
	"net"
	"os"
)

const bufferSize int64 = 1 << 16

func main() {
	raddr, err := net.ResolveTCPAddr("tcp", "127.0.0.1:4242")
	if err != nil {
		log.Fatalln(err)
	}
	conn, err := net.DialTCP("tcp", nil, raddr)
	if err != nil {
		log.Fatalln(err)
	}
	go func() {
		io.CopyBuffer(conn, os.Stdin, make([]byte, bufferSize))
		conn.CloseWrite()
	}()
	io.CopyBuffer(os.Stdout, conn, make([]byte, bufferSize))
	conn.Close()
}
