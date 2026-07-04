# lab_presence
An app that visualizes laboratory occupancy status.

- hub: Maintain status in the laboratory.
- client: Someone checking presence status from outside.
- remote: Someone who changes their presence status from an external location
- NFC: Update your presence status in the laboratory.
- slint: Display presence status in the laboratory.
- webserver: Ensuring redundancy for hub and cloud presence status, and serving as a gateway for external communication.

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
## get presence(for slint or sync between the webserver and the hub)

|item|spec|
|-|-|
|http method|get|
|end point|/api/v1/presence/{userId}/status|
|auth|Authorization: Bearer <token>|

<token> is a hashed onetime device-specific access token.

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
    "presence": string,
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

## change presence remotely

|item|spec|
|-|-|
|http method|put|
|end point|/api/v1/presence/{userId}/status|
|auth|Authorizaiton: Bearer <token>|

<token> is a hashed onetime device-specific access token.

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

## login(to get onetime device-specific access token)

|item|spec|
|-|-|
|http method|post|
|end point|/api/v1/login|
|auth|none|

### Request

Request parameters

|Name|Type|Description|
|-|-|-|
|device_id|string|Required: device id|
|device_secret|string|Required: device secret(like refresh token)|

```json
{
    "device_id": "",
    "device_secret": ""
}
```

### Response

|Status|Description|
|-|-|
|200|The presence state for this device.|
|403|You are not allowed to get onetime access token.|
|404|There is no presence state for this device. This device may not exist or isn't exposing presence information to you.|

#### 200
```json
{
    "access_token": "",
    "expires_in": 3600
}
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
    "error": "requested device was not found."
}
```

---

# another presence updete system can add!
If you have right to look lab's internet ARP table, you can use it.
