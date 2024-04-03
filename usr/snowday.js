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
        });
        console.log(o);

    }
}