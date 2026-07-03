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

# specs
## NFC sends toggle presence(maybe mean entering and leaving the lab)

|item|spec|
|-|-|
|http method|put|
|end point|/api/v1/presence/toggle|
|auth|Authorization: Beare <token>|

### Request
```json
{
    "precense_toggle": true,
    "timestamp": unixtime
}
```
The <token> is created by adding the user ID, salt, and the pepper held by the NFC sender to the hash.

### Response
|Status|Description|
|-|-|
|200|the new presence state was set.|
|404|user or nfc device is not found|

#### 200
```json
{}
```

#### 404
```json
{
    "errorcode": "Not Found",
}
```


## get precense(for slint or sync between the webserver and the hub)

|item|spec|
|-|-|
|http method|get|
|end point|/api/v1/presence/{userId}/status|
|auth|Authorization: Beare <token>|

<token> is a device-specific access token.

### Request

Request parameters

|Name|Type|Description|
|-|-|-|
|userId|string|Required: The user whose presence state to get|

### Response

|Status|Description|
|-|-|
|200|The presence state for this user.|
|403|You are not allowed to see this user’s presence status.|
|404|There is no presence state for this user. This user may not exist or isn’t exposing presence information to you.|

#### 200
```json
{
    "precense": string,
    "status_msg": string,
    "timestamp": unixtime
}
```
#### 403
```json
{
  "errcode": "M_FORBIDDEN",
  "error": "You are not allowed to see their presence"
}
```
#### 404
```json
{
  "errcode": "M_UNKNOWN",
  "error": "An unknown error occurred"
}
```

## change precense remotely

|item|spec|
|-|-|
|http method|put|
|end point|/api/v1/presence/{userId}/status|
|auth|Authorizaiton: Beare <token>|

<token is a device-specific access token.

### Request

Request parameters

|Name|Type|Description|
|-|-|-|
|userId|string|Required: The user whose presence state to get|

### Response

|Status|Description|
|-|-|
|200|The presence state for this user.|
|403|You are not allowed to see this user’s presence status.|
|404|There is no presence state for this user. This user may not exist or isn’t exposing presence information to you.|

#### 200
```json
{}
```
#### 403
```json
{
  "errcode": "M_FORBIDDEN",
  "error": "authorization failed",
}
```
#### 404
```json
{
  "errcode": "M_NOT_FOUND",
  "error": "requested user was not found."
}
```
