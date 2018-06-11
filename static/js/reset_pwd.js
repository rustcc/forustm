$(".change_password :button").click(function (event) {
    event.preventDefault();
    $(".text-danger").remove();
    var re_password = $("#re_password").val();
    var new_password = $("#new_password").val();
    var cookie = window.location.pathname.split("/")[2];
    if (new_password.length < 5) {
        $("#new_password").after("<span class='text-danger' style='display: block'>密码长度小于5</span>")
    } else if (new_password !== re_password) {
        $("#re_password").after("<span class='text-danger' style='display: block'>两次密码不一致</span>")
    } else {
        $.ajax({
            url: "/api/v1/user/reset_pwd",
            type: "post",
            dataType: "json",
            data: JSON.stringify({"password": new_password, "cookie": cookie}),
            headers: {"Content-Type": "application/json"},
            success: function (res) {
                if (res.status) {
                    window.location = "/home"
                } else {
                    $("#re_password").after("<span class='text-danger' style='display: block'>" + res.error + "</span>")
                }
            }
        });
    }
});