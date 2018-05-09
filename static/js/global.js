// TAB in <textarea>
$(document).delegate('textarea', 'keydown', function (e) {
    var keyCode = e.keyCode || e.which;

    if (keyCode == 9) {
        e.preventDefault();
        var start = this.selectionStart;
        var end = this.selectionEnd;

        // set textarea value to: text before caret + tab + text after caret
        $(this).val($(this).val().substring(0, start)
            + "\t"
            + $(this).val().substring(end));

        // put caret at right position again
        this.selectionStart = this.selectionEnd = start + 1;
    }
});

function Page() {
    this.page = 1;
    this.update = function (page) {
        this.page = page
    };
}

var PAGE = new Page();

function checkLoggedIn(run_if_logged_in) {
    var role = $(".role").data("role");
    if (role !== undefined && role === -1) {
        localStorage.setItem("redirect", location.pathname);
        window.location.pathname = '/login';
        return;
    }

    if (window.user !== undefined) {
        run_if_logged_in();
        return;
    }

    $.getJSON("/api/v1/user/view", function (response) {
        if (response.status === false) {
            // need login
            localStorage.setItem("redirect", location.pathname);
            window.location.pathname = '/login';
        } else {
            // logged in
            // save global user data
            window.user = response.data;
            run_if_logged_in();
        }
    });
}

function parseQueryStringToDictionary(queryString) {
    var dictionary = {};

    // remove the '?' from the beginning of the
    // if it exists
    if (queryString.indexOf('?') === 0) {
        queryString = queryString.substr(1);
    }

    // Step 1: separate out each key/value pair
    var parts = queryString.split('&');

    for (var i = 0; i < parts.length; i++) {
        var p = parts[i];
        // Step 2: Split Key/Value pair
        var keyValuePair = p.split('=');

        // Step 3: Add Key/Value pair to Dictionary object
        var key = keyValuePair[0];
        var value = keyValuePair[1];

        // decode URI encoded string
        value = decodeURIComponent(value);
        value = value.replace(/\+/g, ' ');

        dictionary[key] = value;
    }

    // Step 4: Return Dictionary Object
    return dictionary;
}

Date.prototype.Format = function (fmt) { //author: meizz 
    var o = {
        "M+": this.getMonth() + 1, //月份 
        "d+": this.getDate(), //日 
        "h+": this.getHours(), //小时 
        "m+": this.getMinutes(), //分 
        "s+": this.getSeconds(), //秒 
        "q+": Math.floor((this.getMonth() + 3) / 3), //季度 
        "S": this.getMilliseconds() //毫秒 
    };
    if (/(y+)/.test(fmt)) {
        fmt = fmt.replace(RegExp.$1, (this.getFullYear() + "").substr(4 - RegExp.$1.length));
    }
    for (var k in o) {
        if (new RegExp("(" + k + ")").test(fmt)) {
            fmt = fmt.replace(RegExp.$1, (RegExp.$1.length == 1) ? (o[k]) : (("00" + o[k]).substr(("" + o[k]).length)));
        }
    }
    return fmt;
};

Date.prototype.LocalFormat = function () {
    //return new Date(this.getTime() + 3600000 * 8).Format("yyyy-MM-dd hh:mm:ss");
    return this.Format("yyyy-MM-dd hh:mm:ss");
};

// reslove iso date not endwith 'Z' cause display incorrect
var _Date = Date;
Date = function () {
    var time = arguments[0];
    var isoReg = /^\d{4}\-\d{1,2}\-\d{1,2}T\d{1,2}\:\d{1,2}\:\d{1,2}\.\d*$/;
    if (time && isoReg.test(time)) {
        arguments[0] = time + 'Z';
    }
    var args = [].slice.call(arguments);
    args.unshift(_Date);
    if (this instanceof Date) {
        return new (_Date.bind.apply(_Date, args))();
    }
    return (_Date.bind.apply(_Date, args))();
}
Date.prototype = _Date.prototype;
Date.UTC = _Date.UTC;
Date.now = _Date.now;
Date.parse = _Date.parse;


function ctrlEnterThen($DOMs, callback) {
    $DOMs.keydown(function (e) {
        if (e.ctrlKey && e.keyCode === 13) {
            callback();
        }
    });
}

