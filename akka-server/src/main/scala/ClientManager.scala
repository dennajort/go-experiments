import akka.actor.{Actor, ActorLogging, ActorRef, Props, Terminated}
import akka.util.ByteString

import scala.collection.immutable.{List, Set}

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
      clients.foreach { _ ! ClientWriter.SendMessage(msg) }
      context become waitingClients(clients, reader, clients, List.empty)
  }

  def waitingClients(
                  clients: Set[ActorRef],
                  reader: ActorRef,
                  notAnswered: Set[ActorRef],
                  queue: List[(ActorRef, ByteString)]
                ): Receive = {
    case ClientJoin(connection) =>
      val writer = context actorOf Props(classOf[ClientWriter], connection)
      context watch writer
      log.info(s"New client, now have ${clients.size + 1} clients")
      context actorOf Props(classOf[ClientReader], self, connection, writer)
      context become waitingClients(clients + writer, reader, notAnswered, queue)

    case Terminated(client) =>
      log.info(s"Client left, now have ${clients.size - 1} clients")
      processAnswer(clients - client, reader, notAnswered - client, queue)

    case ClientClosed(client) =>
      log.info(s"Client left, now have ${clients.size - 1} clients")
      context unwatch client
      processAnswer(clients - client, reader, notAnswered - client, queue)

    case ClientWriter.MessageSent =>
      log.debug(s"Client sent message")
      val client = sender()
      processAnswer(clients, reader, notAnswered - client, queue)

    case BroadcastMessage(msg) =>
      val nextReader = sender()
      context become waitingClients(clients, reader, notAnswered, queue :+ (nextReader, msg))
  }

  def processAnswer(
                     clients: Set[ActorRef],
                     reader: ActorRef,
                     notAnswered: Set[ActorRef],
                     queue: List[(ActorRef, ByteString)]
                   ): Unit = {
    if (notAnswered.isEmpty) {
        log.debug(s"All writers sent message")
        reader ! MessageBroadcasted
        queue match {
          case (next, msg) :: rest =>
            log.debug(s"Broadcasting Message to ${clients.size} clients")
            clients.foreach { _ ! ClientWriter.SendMessage(msg) }
            context become waitingClients(clients, next, clients, rest)
          case Nil =>
            log.debug("No more message to broadcast")
            context become idle(clients)
        }
    } else {
        log.debug(s"Still ${notAnswered.size} clients didn't sent message")
        context become waitingClients(clients, reader, notAnswered, queue)
    }
  }
}
