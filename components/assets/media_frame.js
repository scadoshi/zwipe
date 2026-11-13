// A framed image or video (components' MediaFrame) stays hidden over its
// skeleton until it can paint its first frame; this marks it ready then, or
// on an error so a broken file never pulses forever. load, loadeddata and
// error don't bubble, so the listener captures them, and media already done
// before it was added is marked at once and again when the page has parsed.
(function () {
    var ready = function (el) {
        if (el instanceof Element && el.parentElement && el.parentElement.classList.contains("media-frame")) {
            el.setAttribute("data-ready", "");
        }
    };
    ["load", "loadeddata", "error"].forEach(function (type) {
        document.addEventListener(type, function (e) { ready(e.target); }, true);
    });
    var done = function () {
        document.querySelectorAll(".media-frame > img, .media-frame > video").forEach(function (el) {
            if (el.tagName === "IMG" ? el.complete : el.readyState >= 2) ready(el);
        });
    };
    done();
    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", done);
    }
})();
