Yet Another Markdown Server

This is the server for my website to serve my pages written in markdown.
Some features will be left to a reverse proxy for now such as tls and
caching of generated pages. This is mainly an experiment and learning
experience to see what it is like to implement a web server with a few
features that are sort of difficult to find together on other servers.

Features:
 [ ] Render Markdown to serve as html
 [ ] Combine rendered html with provided css
 [ ] Serve markdown as .html and hide the original format of pages
 [ ] Fetch updated pages via git upon the reciept of a webhook

Caddy has almost all the above features but it serves the rendered markdown
pages as .md and not .html, making future changes in format annoying and
possibly causing other problems.

