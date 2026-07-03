# lab_presence
An app that visualizes laboratory occupancy status.

- local: Maintain status in the laboratory.
- client: Someone checking presence status from outside.
- remote: Someone who changes their presence status from an external location
- NFC: Update your presence status in the laboratory.
- slint: Display presence status in the laboratory.
- webserver: Ensuring redundancy for local and cloud presence status, and serving as a gateway for external communication.

```mermaid
flowchart TD
    hub --> webserver
    client --> webserver
    NFC --> hub
    webserver --> hub
    webserver --> database
    slint --> hub
    remote --> webserver

    subgraph public-cloud
        database
        webserver
    end
    subgraph Lab
        hub
        NFC
        slint
    end
```

API Refarence
https://spec.matrix.org/v1.18/client-server-api/#client-behaviour-8
However, to determine which state was the most recent when the Lab network was disconnected, it seems necessary to also send the timestamp of when that status was saved.
