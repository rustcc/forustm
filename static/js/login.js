"use strict";
$().ready(function () {
    document.onkeydown = function (e) {
        var ev = document.all ? window.event : e;
        if (ev.keyCode === 13) {
            if ($("#login_form").css("display") === 'block') {
                login()
            } else if ($("#register_form").css("display") === 'block'){
                register()
            } else {
                reset_account()
            }
        }
    }
});

$("#register_btn").click(function () {
    $("#register_form").css("display", "block");
    $("#login_form").css("display", "none");
    $("#reset_form").css("display", "none");
    $(".text-danger").remove();
    $("input[type!='button']").val("");
    $("#register_account").focus();
});

$(".back_btn").click(function () {
    $("#register_form").css("display", "none");
    $("#login_form").css("display", "block");
    $("#reset_form").css("display", "none");
    $(".text-danger").remove();
    $("input[type!='button']").val("");
    $("#login_account").focus();
});

$("#forget").click(function () {
    $("#register_form").css("display", "none");
    $("#login_form").css("display", "none");
    $("#reset_form").css("display", "block");
    $(".text-danger").remove();
    $("input[type!='button']").val("");
    $("#reset_account").focus();
});

$("#login").click(function (event) {
        event.preventDefault();
        login()
    }
);

$("#sign_up").click(function (event) {
        event.preventDefault();
        register()
    }
);

$("#reset_btn").click(function (event) {
    event.preventDefault();
    reset_account()
});

function login() {
    var account = $("#login_account").val();
    var password = $("#login_password").val();
    var remember = $(".checkbox").children().is(':checked');
    $(".text-danger").remove();
    if (emailVerification(account) && password.length >= 5) {
        $.ajax({
            url: "/api/v1/user/login",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "account": account, "password": password, "remember": remember }),
            headers: { 'Content-Type': 'application/json' },
            success: function (res) {
                if (res.status) {
                    window.location = "/home"
                } else {
                    if (res.error === "NotFound") {
                        $("#login").prev().before("<span class='text-danger'>用户被锁定或未创建</span>")
                    } else {
                        $("#login").prev().before("<span class='text-danger'>用户名或密码错误</span>")
                    }
                }
            }
        })
    } else {
        $("#login").prev().before("<span class='text-danger'>Email无效或密码小于5位</span>")
    }
}

function register() {
    var account = $("#register_account").val();
    var password = $("#register_password").val();
    var nickname = $("#nickname").val();
    $(".text-danger").remove();
    if (emailVerification(account) && password.length >= 5 && nickname !== "") {
        $.ajax({
            url: "/api/v1/user/sign_up",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "account": account, "password": password, "nickname": nickname }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    window.location = "/home"
                } else {
                    $("#sign_up").prev().before("<span class='text-danger'>用户已创建</span>")
                }
            }
        })
    } else {
        $("#sign_up").prev().before("<span class='text-danger'>Email无效或密码小于5位</span>")
    }
}

function reset_account() {
    var account = $("#reset_account").val();
    $(".text-danger").remove();
    if (account !== "") {
        $.ajax({
            url: "/api/v1/user/reset_pwd",
            type: "post",
            dataType: "json",
            data: JSON.stringify({ "account": account }),
            headers: { "Content-Type": "application/json" },
            success: function (res) {
                if (res.status) {
                    window.location = "/login"
                } else {
                    $("#reset_btn").prev().before("<span class='text-danger'>用户不存在</span>")
                }
            }
        })
    } else {
        $("#reset_btn").prev().before("<span class='text-danger'>account 不能为空</span>")
    }
}

function emailVerification(email) {
    var reg_email = /^([a-zA-Z0-9]+[_|\_|\.]?)*[a-zA-Z0-9]+@([a-zA-Z0-9]+[_|\_|\.]?)*[a-zA-Z0-9]+\.[a-zA-Z]{2,3}$/;
    return reg_email.test(email);
}