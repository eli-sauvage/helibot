let ajt = document.getElementById("ajt");
let form = document.getElementById("form");
let envoi = document.getElementById("envoi");
let q = document.getElementById("quest");
let msg = document.getElementById("msg");
let index = 2; //nb de reponses
ajt.addEventListener("click", () => {
  if (index >= 36) {
    msg.textContent = "pas + de 36 reponses";
    setTimeout(() => {
      msg.textContent = "";
    }, 1000);
    return;
  }
  let div = document.createElement("div");
  div.id = index;
  let inp = document.createElement("input");
  inp.type = "text";
  inp.name = "name";
  inp.required = "true";
  inp.className = "rep";
  div.appendChild(inp);
  let del = document.createElement("button");
  del.textContent = "suppr";
  div.appendChild(del);
  const cindex = index;
  del.addEventListener("click", () => {
    console.log(cindex);
    console.log(document.getElementById(cindex));
    form.removeChild(document.getElementById(cindex));
  });
  let br = document.createElement("br");
  div.appendChild(br);
  index++;
  form.insertBefore(div, ajt);
});

envoi.addEventListener("click", () => {
  // fetch("https://77.151.84.172:2832/poll", {method: "POST"})
  let question = q.value;
  let rep = Array.from(document.getElementsByClassName("rep")).map(
    e => e.value
  );
  if (question == "" || rep.includes("")) return;
    fetch("https://77.151.84.172:2832/poll", {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ quest: question, rep: rep, every: document.getElementById("mention").checked }) 
  })
    console.log(JSON.stringify({ quest: question, rep: rep, every: document.getElementById("mention").checked }) )
});
