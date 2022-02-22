(async ()=>{
	let res = await fetch("./points")
	let points = await res.json()
	points.sort((a,b)=>{
		if(a.Points>b.Points)return -1
		if(a.Points<b.Points)return 1

		if(a.AncienScore>b.AncienScore)return -1
		if(a.AncienScore<b.AncienScore)return 1


	})
	let table = document.getElementById("table")
	points.forEach(pt=>{
	    let tr = document.createElement("tr")
	    let nom = document.createElement("td")
	    nom.textContent = pt.Name
	    let score = document.createElement("td")
	    score.textContent = pt.Points
	    let ancien = document.createElement("td")
	    ancien.textContent = pt.AncienScore
	    tr.appendChild(nom)
	    tr.appendChild(score)
	    tr.appendChild(ancien)
            table.appendChild(tr)
	})
	console.log(points)
})()
