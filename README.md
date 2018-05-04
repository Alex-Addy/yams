## Yet Another Markdown Server

### WARNING!
This is just a personal experiment meant for my personal use, hence the
dumb name and reliance on possibly unstable crates. I make no guarantees
about the quality of code or it even working for anyone but myself.

### Description

This is the server for my website to serve my pages written in markdown.
Some features will be left to a reverse proxy for now such as tls and
caching of generated pages. This is mainly an experiment and learning
experience to see what it is like to implement a web server with a few
features that are sort of difficult to find together on other servers.

Features:
 - [x] Render Markdown to serve as html
 - [x] Combine rendered html with provided css
 - [x] Serve markdown as .html and hide the original format of pages
 - [x] Fetch updated pages via git upon the reciept of a webhook
 - [ ] Cache resources in memory
 - [ ] Correctly work with cach headers
 - [ ] Enable TLS for https

Caddy has almost all the above features but it serves the rendered markdown
pages as .md and not .html, making future changes in format annoying if
stable links are desired.

