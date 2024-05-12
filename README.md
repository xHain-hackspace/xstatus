# xStatus - serving the spaceapi for xHain

This is a simple rust server which uses [actix-web](https://actix.rs/) and builds on [spaceapi-rs](https://github.com/spaceapi-community/spaceapi-rs).

To retrieve the status query the `/spaceapi/14/status` endpoint using `GET`.

To set the state `POST` json encoded data to /spaceapi/14/status/state for example:

```
'{"open": false, "message": "closed"}'
```


