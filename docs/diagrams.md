```plantuml
@startuml
hide empty members

package transport {
    abstract Connector {
        type Inbound
        type Outbound
    }
    abstract Transport {
        type Connection
        type Error
    }
    abstract Receiver {
        type Error
    }
    abstract Sender {
        type Error
    }
    abstract Inbound {
        type InboundQueue
    }
    abstract Outbound {
        type OutboundQueue
    }

    class Message
    class Part
    enum Kind

    Connector -d-> Inbound
    Connector -d-> Outbound

    Inbound -u-|> Transport
    Outbound -u-|> Transport

    Inbound --> Receiver
    Outbound --> Sender

    Sender --> Message
    Receiver --> Message

    Message *-d--> Part
    Part *-d-> Kind
}

package imap {
    class Imap
    class ImapConnection

    Imap -d-> ImapConnection
}

Imap -r-|> Inbound
ImapConnection -r-|> Receiver

package smtp {
    class Smtp
    class SmtpConnection

    Smtp -d-> SmtpConnection
}

Smtp -l-|> Outbound
SmtpConnection -l-|> Sender

package service {
    class Request
    class Response
    abstract Service

    Service -u-> Request
    Service -d-> Response
}

Request *-l-> Message
Response *-l-> Part

class ConnectionHandler<T>

ConnectionHandler *-d-> Transport

@enduml
```





