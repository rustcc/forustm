"use strict";

function changeActive($node) {
    $(".active").removeClass("active");
    $node.addClass("active");
}

$("#information_btn").click(function() {
    changeActive($(this));
    $(".information").css("display", "block");
    $(".modify").css("display", "none");
    $(".change_password").css("display", "none");
});

$("#modify_btn").click(function () {
    changeActive($(this));
    $(".information").css("display", "none");
    $(".modify").css("display", "block");
    $(".change_password").css("display", "none");
    getInfo()
});

$("#change_password_btn").click(function () {
    changeActive($(this));
    $(".information").css("display", "none");
    $(".modify").css("display", "none");
    $(".change_password").css("display", "block");
    clearPassword()
});

$(".change_password :button").click(function (event) {
    event.preventDefault();
    $(".text-danger").remove();
    var old_password = $("#old_password").val();
    var re_password = $("#re_password").val();
    var new_password = $("#new_password").val();
    if (old_password.length < 5) {
        $("#old_password").after("<span class='text-danger' style='display: block'>密码长度小于5</span>")
    } else if (new_password.length < 5) {
        $("#new_password").after("<span class='text-danger' style='display: block'>密码长度小于5</span>")
    } else if (new_password !== re_password) {
        $("#re_password").after("<span class='text-danger' style='display: block'>两次密码不一致</span>")
    } else {
        $.ajax({
            url: "/api/v1/user/change_pwd",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "old_password": old_password, "new_password": new_password }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    window.location = "/home"
                } else {
                    $("#re_password").after("<span class='text-danger' style='display: block'>密码错误</span>")
                }
            }
        });
    }
});

$(".modify :button").click(function (event) {
    event.preventDefault();
    var nickname = $("#nickname").val().replace(/(^\s*)|(\s*$)/g, "");
    var say = $("#say").val();
    var avatar = $("#avatar").val().replace(/(^\s*)|(\s*$)/g, "");
    var wx_openid = $("#wx_openid").val();
    $(".text-danger").remove();
    if (nickname !== "") {
        $.ajax({
            url: "/api/v1/user/edit",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "nickname": nickname, "say": say, "avatar": avatar, "wx_openid": wx_openid }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    window.location = "/home"
                }
            }
        });
    } else {
        $(this).before("<span class='text-danger' style='display: block'>昵称为空</span>")
    }
});

function getInfo() {
    $("#nickname").val($(".nickname").text());
    $("#say").val($(".say").text());
    $("#wx_openid").val($(".wx_openid").text());
    $("#avatar").val($("#path")[0].src)
}

function clearPassword() {
    $("#old_password").val("");
    $("#re_password").val("");
    $("#new_password").val("");
    $("#old_password").focus();
}
