async function test() {
    let postReq = {
        contents: document.getElementById('in').value,
    }
    console.log("A: " + postReq);
    let url = '/waow.html'
    let password = "transrights";
    o = await fetch(url, {
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json',
            'Password': password
        },
        body: JSON.stringify(postReq)
    }).then(response => {
        console.log(response);
    });

}
async function test2() {
    let url = '/waow.html'
    let password = "transrights";
    o = await fetch(url, {
        method: 'DELETE',
        headers: {
            'Content-Type': 'application/json',
            'Password': password
        },
    }).then(response => {
        console.log(response);
    });
}