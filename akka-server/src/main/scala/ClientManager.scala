import akka.actor.{Actor, ActorLogging, ActorRef, Props, Terminated}
import akka.util.ByteString

import scala.collection.immutable.Set

object ClientManager {
  case class ClientJoin(connection: ActorRef)
  case class ClientClosed(writer: ActorRef)
  case class BroadcastMessage(msg: ByteString)
  case class MessageBroadcasted()
}

class ClientManager extends Actor with ActorLogging {
  import ClientManager._

  def receive: Receive = idle(Set.empty)

  def idle(clients: Set[ActorRef]): Receive = {
    case ClientJoin(connection) =>
      val writer = context actorOf Props(classOf[ClientWriter], connection)
      context watch writer
      log.info(s"New client, now have ${clients.size + 1} clients")
      context actorOf Props(classOf[ClientReader], self, connection, writer)
      context become idle(clients + writer)

    case Terminated(client) =>
      log.info(s"Client left, now have ${clients.size - 1} clients")
      context become idle(clients - client)

    case ClientClosed(client) =>
      log.info(s"Client left, now have ${clients.size - 1} clients")
      context unwatch client
      context become idle(clients - client)

    case BroadcastMessage(msg) =>
      log.debug(s"Broadcasting Message to ${clients.size} clients")
      val reader = sender()
      context actorOf Props(classOf[BroadcastTask], clients, reader, msg)
  }
}
