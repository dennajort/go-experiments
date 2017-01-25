from typing import List
import asyncio

loop = asyncio.get_event_loop()

clients = []  # type: List[EchoProtocol]


class EchoProtocol(asyncio.Protocol):
    def __init__(self):
        self.transport = None  # type: asyncio.Transport

    def connection_made(self, transport: asyncio.Transport):
        self.transport = transport
        clients.append(self)
        print("Client connected, %d clients" % len(clients))

    def data_received(self, data: bytes):
        for client in clients:
            client.transport.write(data)

    def connection_lost(self, exc: Exception):
        clients.remove(self)
        if exc is not None:
            print(exc)
        print("Client disconnected, %d clients" % len(clients))


server = loop.run_until_complete(
    loop.create_server(EchoProtocol, port=4242)
)

print("Started")
try:
    loop.run_forever()
except:
    pass

server.close()
loop.run_until_complete(server.wait_closed())
loop.close()
