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


## Changes
### 2.2.4
* Grpc Client now have timeouts
* Backgrounds are implemented using timers which means now they have one minute timeout in case of long running tasks;
* Added Metric - topic size in memory
* Highlited PageId within MessageID on UI

### 2.2.5
* Pages Support
* GC works as fast as it can
* Added Visualisation - how many messages are on the delivery
* UI Shows amount of Sessions
* Bug Fixed - immediate persistence made to send a lot of data to console.

### 2.2.6
* Immediately persist case is signle threaded
* Added ability to send messages to persist uncompressed way (Settings Parameter PersistCompressed)
* BugFIX: When we delete a queue - we remove topic_queue_size from prometheus

### 2.2.7-rc01
* Updated Library versions