let o;
async function calculate() {
    let zipcode = document.getElementById("zipcode");
    if (!(/(^\d{5}$)|(^\d{5}-\d{4}$)/.test(zipcode.value))) {
        console.log("Invalid Zipcode");
    } else {
        let postReq = {
            zipcode: zipcode.value
        }
        console.log(postReq);
        let url = '/snowday'
        o = await fetch(url, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(postReq)
        }).then(response => response.json());
        console.log(o);
        let list = document.getElementById('lsit');
        list.innerHTML = "";
        for (i = 0; i < o.length; ++i) {
            var li = document.createElement('li');
            var button = document.createElement('button');
            button.innerText = i + 1 + ": " + o[i][0] + "\n";
            button.addEventListener('click', () => selectLocation(i));
            li.appendChild(button);
            list.appendChild(li);
        }
    }
}
async function selectLocation(inpit) {
    console.log("ASDAS: " + o);
    if (inpit == undefined) {
        inpit = document.getElementById("location");
        console.log("TEST")
    }
    console.log("HELP ME: " + inpit);
    let postReq = {
        lat: o[inpit - 1][1],
        lng: o[inpit - 1][2]
    }
    console.log(postReq);
    console.log("for: " + o[inpit - 1][0]);
    let url = '/snowday_latlong'
    e = await fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(postReq)
    }).then(response => response.json());
    console.log(e);
    let resilt = document.getElementById('result');
    resilt.innerText = e + "%";
}