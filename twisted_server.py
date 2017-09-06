from twisted.internet import protocol, reactor, endpoints

clients = []  # type: List[Echo]


class Echo(protocol.Protocol):
    def connectionMade(self):
        clients.append(self)
        print("Client connected, %d clients" % len(clients))

    def connectionLost(self, reason):
        clients.remove(self)
        if reason is not None:
            print(reason)
        print("Client disconnected, %d clients" % len(clients))

    def dataReceived(self, data):
        for client in clients:
            client.transport.write(data)


class EchoFactory(protocol.Factory):
    def buildProtocol(self, addr):
        return Echo()


endpoints.serverFromString(reactor, "tcp:4242").listen(EchoFactory())
print("Started")
reactor.run()