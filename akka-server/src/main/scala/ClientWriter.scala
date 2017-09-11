import akka.actor.{Actor, ActorLogging, ActorRef}
import akka.util.ByteString
import scala.collection.immutable.List

object ClientWriter {
  case class SendMessage(msg: ByteString)
  case class MessageSent()
}

class ClientWriter(connection: ActorRef) extends Actor with ActorLogging {
  import ClientWriter._
  import akka.io.Tcp._

  case object Ack extends Event

  def receive: Receive = idle()

  def idle(): Receive = {
    case SendMessage(msg) =>
      val receiver = sender()
      connection ! Write(msg, Ack)
      context become waitingAck(receiver, Nil)
  }

  def waitingAck(waiter: ActorRef, queue: List[(ActorRef, ByteString)]): Receive = {
    case SendMessage(msg) =>
      val receiver = sender()
      context become waitingAck(waiter, queue :+ (receiver, msg))
    case Ack =>
      waiter ! MessageSent
      queue match {
        case (next, msg) :: rest =>
          connection ! Write(msg, Ack)
          context become waitingAck(next, rest)
        case Nil =>
          context become idle()
      }
    case CommandFailed =>
      context stop self
  }
}
