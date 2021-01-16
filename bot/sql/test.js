var mysql = require('mysql');

var db = mysql.createConnection({
    host: '77.151.84.172', 
    port:'3306',
    user:'bot',
    password: require("../../token").sql,
    database: "helibot"
 });

db.connect(function(err) {

    if (err) throw err;

    console.log("Connecté à la base de données MySQL!");

    db.query("SELECT max(id) FROM Session Where User = 280769421604290560", function (err, result) {

        if (err) throw err;

        console.log(result);

        });

});