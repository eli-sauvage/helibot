let ajt = document.getElementById("ajt");
let form = document.getElementById("form");
let envoi = document.getElementById("envoi");
let q = document.getElementById("quest");
let msg = document.getElementById("msg");
let index = 2; //nb de reponses
envoi.addEventListener("click", () => {
  console.log("click")
  fetch("https://77.151.84.172:2832/poll", {
    method: "POST",
    body: JSON.stringify({ "quest": "sdf", "rep": ["sdfg", "gsdfg"], "every": false })
  })
})