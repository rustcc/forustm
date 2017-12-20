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
