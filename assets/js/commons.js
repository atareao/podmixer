"use strict";
!(function() {
    var e = {
        _scheme: "auto",
        change: {
            light: '<i class="fa-solid fa-moon"></i>',
            dark: '<i class="fa-solid fa-sun"></i>',
        },
        buttonsTarget: ".theme-switcher",
        localStorageKey: "picoPreferredColorScheme",
        init() {
            (this.scheme = this.schemeFromLocalStorage), this.initSwitchers();
        },
        get schemeFromLocalStorage() {
            return void 0 !== window.localStorage &&
                null !== window.localStorage.getItem(this.localStorageKey)
                ? window.localStorage.getItem(this.localStorageKey)
                : this._scheme;
        },
        get preferredColorScheme() {
            return window.matchMedia("(prefers-color-scheme: dark)").matches
                ? "dark"
                : "light";
        },
        initSwitchers() {
            document.querySelectorAll(this.buttonsTarget).forEach((e) => {
                e.addEventListener(
                    "click",
                    () => {
                        "dark" == this.scheme
                            ? (this.scheme = "light")
                            : (this.scheme = "dark");
                    },
                    !1,
                );
            });
        },
        addButton(e) {
            var t = document.createElement(e.tag);
            (t.className = e.class),
                document.querySelector(e.target).appendChild(t);
        },
        set scheme(e) {
            "auto" == e
                ? "dark" == this.preferredColorScheme
                    ? (this._scheme = "dark")
                    : (this._scheme = "light")
                : ("dark" != e && "light" != e) || (this._scheme = e),
                this.applyScheme(),
                this.schemeToLocalStorage();
        },
        get scheme() {
            return this._scheme;
        },
        applyScheme() {
            document.querySelector("html").setAttribute("data-theme", this.scheme),
                document.querySelectorAll(this.buttonsTarget).forEach((e) => {
                    var t =
                        "dark" == this.scheme ? this.change.dark : this.change.light;
                    (e.innerHTML = t),
                        e.setAttribute("aria-label", t.replace(/<[^>]*>?/gm, ""));
                });
        },
        schemeToLocalStorage() {
            void 0 !== window.localStorage &&
                window.localStorage.setItem(this.localStorageKey, this.scheme);
        },
    },
        t = {
            _state: "closed-on-mobile",
            toggleLink: document.getElementById("toggle-docs-navigation"),
            nav: document.querySelector("main > aside > nav"),
            init() {
                this.onToggleClick();
            },
            onToggleClick() {
                this.toggleLink.addEventListener(
                    "click",
                    (e) => {
                        e.preventDefault(),
                            "closed-on-mobile" == this.state
                                ? (this.state = "open")
                                : (this.state = "closed-on-mobile"),
                            this.nav.removeAttribute("class"),
                            this.nav.classList.add(this.state);
                    },
                    !1,
                );
            },
            get state() {
                return this._state;
            },
            set state(e) {
                this._state = e;
            },
        };
    e.addButton({
        tag: "BUTTON",
        class: "contrast switcher theme-switcher",
        target: "body",
    }),
        e.init(),
        t.init();
})();
