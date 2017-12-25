// TAB in <textarea>
$(document).delegate('textarea', 'keydown', function(e) {
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

function checkLoggedIn(run_if_logged_in){
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

    $.getJSON("/api/v1/user/view", function(response){
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
    if (/(y+)/.test(fmt)) fmt = fmt.replace(RegExp.$1, (this.getFullYear() + "").substr(4 - RegExp.$1.length));
    for (var k in o)
    if (new RegExp("(" + k + ")").test(fmt)) fmt = fmt.replace(RegExp.$1, (RegExp.$1.length == 1) ? (o[k]) : (("00" + o[k]).substr(("" + o[k]).length)));
    return fmt;
}

Date.prototype.LocalFormat = function () {
    return new Date(this.getTime() + 3600000 * 8).Format("yyyy-MM-dd hh:mm:ss");
}
