import akka.actor.{Actor, ActorLogging, ActorRef, Terminated}
import akka.util.ByteString

import scala.collection.immutable.Set

class BroadcastTask(initClients: Set[ActorRef], caller: ActorRef, msg: ByteString) extends Actor with ActorLogging {

  initClients.foreach { (client) =>
    context watch client
    client ! ClientWriter.SendMessage(msg)
  }

  log.debug(s"Broadcasting Message to ${initClients.size} clients")

  def receive: Receive = waiting(initClients)

  def waiting(clients: Set[ActorRef]): Receive = {
    case Terminated(client) =>
      processAnswer(clients - client)

    case ClientWriter.MessageSent =>
      log.debug(s"Client sent message")
      val client = sender()
      context unwatch client
      processAnswer(clients - client)
  }

  def processAnswer(clients: Set[ActorRef]): Unit = {
    if (clients.isEmpty) {
      log.debug(s"All writers sent message")
      caller ! ClientManager.MessageBroadcasted
      context stop self
    } else {
      log.debug(s"Still ${clients.size} clients didn't sent message")
      context become waiting(clients)
    }
  }
}
