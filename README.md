# WebSocket <-> HTTP Bridge
So, as I've been pondering how I'll be writing some little Web Apps I've noticed that a lot
of languages that work really well for HTTP/REST don't really work for WebSockets. I suppose
one of the bigger examples would be PHP, which works quite alright with the classical
HTTP request/response model but doesn't really have a nice way of doing WebSockets. Mainly
what I want to build here is a WS <-> HTTP Bridge that preserves the CGI model as much
as possible.

## Status
Right now it's just an idea, although hopefully I'll get a working prototype done pretty soon so
that I can test how well/simple the integration into various languages/frameworks actually is.

## Philosophy
This bridge is **NOT meant to be high-performance!** If you want to build a service that needs to handle
millions of WebSockets then using this bridge would probably be a terrible idea.

However, there are a lot of little apps or MVP's that can only dream of having a couple
of hundred simultaneous users and where **simplicity/ease of use** is much more important
that raw performance. If you have a project like this then ws2http might be a nice choice.

## Architecture
The idea is that a small program keeps translating WebSocket events into HTTP request.
That's it.  It should mostly be used like nginx or MariaDB, as in: mostly be installed via a distro's
package manager (or built from source), and then run as a system service (or if you're into docker, run it
in a container).

By building it this way integrating it into various languages/runtimes should be a breeze, since pretty
much any language used for WebDev has some way of serving HTTP requests and doing them on your own.
And if not then raw CGI/curl should suffice, so even a Bash script can do WebSockets!

### Details
This bridge is not meant to connect to clients directly, since then we'd have to implement TLS
and a lot of other complicated things. Rather the idea is to have nginx in front and configure
the URL's that are valid WebSockets and then proxy those requests to ws2http.

You can then configure in ws2http how to route events based on the URL using regexes, it'll then
do a single request for each WebSocket event (Open/Close/Message). Once a connection is established
the connection will get a unique ID and the Url and Cookies are stored in a Map so that we can
add them to any event so that the application can handle things appropriately.
