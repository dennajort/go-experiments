import akka.actor.{Actor, ActorLogging, ActorRef, Props}
import akka.io.{IO, Tcp}
import java.net.InetSocketAddress

import ClientManager.ClientJoin

class ListenSocket(manager: ActorRef) extends Actor with ActorLogging {
  import Tcp._
  import context.system

  IO(Tcp) ! Bind(self, new InetSocketAddress("localhost", 4242), pullMode = true)

  def receive = {
    case Bound(_) =>
      val listener = sender()
      listener ! ResumeAccepting(batchSize = 1)
      context.become(listening(listener))

    case CommandFailed(_) =>
      context stop self
  }

  def listening(listener: ActorRef): Receive = {
    case Connected(_, _) =>
      val connection = sender()
      manager ! ClientJoin(connection)
      listener ! ResumeAccepting(batchSize = 1)
  }
}
