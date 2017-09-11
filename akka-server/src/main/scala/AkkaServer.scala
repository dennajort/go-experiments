import akka.actor.{ActorSystem, Props}
import com.typesafe.config.ConfigFactory


object AkkaServer extends App {
  val customConf = ConfigFactory.parseString("""
  akka {
    loglevel = "INFO"
  }
  """)

  val system = ActorSystem("AkkaServer", ConfigFactory.load(customConf))
  val manager = system.actorOf(Props[ClientManager])
  println(s"Manager: $manager")
  val listener = system.actorOf(Props(classOf[ListenSocket], manager))
  println(s"Listener: $listener")
}