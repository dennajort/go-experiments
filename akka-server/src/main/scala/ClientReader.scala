import akka.actor.{Actor, ActorLogging, ActorRef}
import akka.io.Tcp

class ClientReader(manager: ActorRef, connection: ActorRef, writer: ActorRef) extends Actor with ActorLogging {
  import Tcp._

  case object Ack extends Event

  connection ! Register(self, keepOpenOnPeerClosed = true)
  connection ! ResumeReading

  def receive: Receive = {
    case Received(data) =>
      log.debug("Received Message from socket")
      manager ! ClientManager.BroadcastMessage(data)

    case PeerClosed =>
      log.debug("PeerClosed")
      manager ! ClientManager.ClientClosed(writer)
      context stop self

    case ClientManager.MessageBroadcasted =>
      log.debug("MessageBroadcasted")
      connection ! ResumeReading
  }
}
