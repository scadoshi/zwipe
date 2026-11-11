// Every fetch Response reports a URL. The browser leaves `url` empty on a
// Response that script built (a fetch-wrapping extension, a DevTools
// override), and the app's HTTP client parses the URL of every reply, so an
// empty one throws inside the wasm and kills the call. Such a response
// reports the page's URL instead, which the app never reads; its body still
// decodes, or fails as any bad reply does.
(function () {
    var own = Object.getOwnPropertyDescriptor(Response.prototype, "url");
    if (!own || !own.get) return;
    Object.defineProperty(Response.prototype, "url", {
        configurable: true,
        enumerable: own.enumerable,
        get: function () {
            return own.get.call(this) || location.href;
        },
    });
})();
