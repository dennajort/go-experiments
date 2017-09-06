from gevent import monkey; monkey.patch_all()
from gevent.server import StreamServer
import socket
from threading import Lock

clients = []  # type: List[Echo]
l = Lock()


def connection_handler(sock: socket.socket, addr):
    with l:
        clients.append(sock)
    print("Client connected, %d clients" % len(clients))
    try:
        while True:
            data = sock.recv(2048)
            if data:
                with l:
                    for client in clients:
                        client.sendall(data)
            else:
                break
    finally:
        with l:
            clients.remove(sock)
        print("Client disconnected, %d clients" % len(clients))


server = StreamServer(('0.0.0.0', 4242), handle=connection_handler)
server.serve_forever()
