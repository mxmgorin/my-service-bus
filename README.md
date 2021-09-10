# MY SERVICE BUS

## Run  

You should run my-service-bus-persistence before running my-service-bus

Enusure that environment variable "**HOME**" exists.
It should point to location with **.myservicebus** file!

**.myservicebus** content:
`
GrpcUrl: http://127.0.0.1:7124 // my-service-bus-persistence should run on this url
EventuallyPersistenceDelay: 00:00:05
QueueGcTimeout: 00:00:20
DebugMode: true
MaxDeliverySize: 4194304
`

Install rust: https://www.rust-lang.org/tools/install
execute: **cargo run --release**
