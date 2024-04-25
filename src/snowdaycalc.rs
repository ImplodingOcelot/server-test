pub async fn get_zip_code(zipcode: usize) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let url = format!(
        "http://api.geonames.org/postalCodeLookupJSON?postalcode={}&username=aaaa",
        zipcode
    );
    let response = reqwest::get(&url).await?.text().await?;
    let parsed = json::parse(&response).unwrap();
    let ans: Vec<Vec<String>> = parsed["postalcodes"]
        .members()
        .map(|x| {
            vec![
                x["placeName"].to_string(),
                x["lat"].to_string(),
                x["lng"].to_string(),
            ]
        })
        .collect();
    return Ok(ans);
}
pub async fn get_snow_day_chances(lat: u32, lng: u32) -> Result<f32, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m,apparent_temperature,precipitation_probability,precipitation,snowfall,visibility,wind_speed_10m,&forecast_days=2",
        lat, lng
    );
    /*
    Parematers:
    hourly_temp
    relative_humidity
    dew_point
    apparent_temperature
    precipitation_probability
    precipitation
    snowfall
    visibility
    wind_speed
    */
    let response = reqwest::get(&url).await.unwrap().text().await.unwrap();
    let jsonval = json::parse(&response).unwrap();
    let mut snowdaypoints: f32 = 0f32;
    let mut temp = 0f64;
    let mut temp2;
    // add all snowfall values to snowdaypoints
    for i in 0..48 {
        let temp_2m = jsonval["hourly"]["snowfall"][i].as_f64().unwrap();
        temp += temp_2m;
    }
    if temp > 0f64 {
        println!("Added 3 points for snowfall initial");
        snowdaypoints += 3f32;
    }
    println!("Added {:?} points for snowfall", temp as i32);
    snowdaypoints += (temp as f32) / 2f32;
    temp = 0f64;

    // add average temp of each hour
    temp2 = 1000f64;
    for i in 0..48 {
        let temp_2m = jsonval["hourly"]["temperature_2m"][i].as_f64().unwrap();
        if temp_2m < temp2 {
            temp2 = temp_2m;
        }
        temp += temp_2m;
    }
    temp /= 48f64;
    if temp < 0f64 {
        println!("Added 2 points for average temp initial");
        snowdaypoints += 2f32;
    }
    if temp2 < -8f64 {
        println!(
            "Added 7 points for lowest temp initial and {:?} points for lowest temp",
            (temp2 as f32).abs()
        );
        snowdaypoints += 7f32 + (temp2 as f32).abs();
    }
    for i in 0..48 {
        let temp_2m = jsonval["hourly"]["visibility"][i].as_f64().unwrap();
        if temp_2m < 1000f64 {
            println!("Added 1 point for low visibility");
            snowdaypoints += 1f32;
            if temp_2m < 100f64 {
                println!("Added 5 points for very low visibility");
                snowdaypoints += 5f32;
                if temp_2m < 10f64 {
                    println!("Added 10 points for extremely low visibility");
                    snowdaypoints += 10f32;
                }
            }
        }
    }
    temp = 0f64;
    temp2 = 0f64;
    for i in 0..48 {
        let temp_2m = jsonval["hourly"]["wind_speed_10m"][i].as_f64().unwrap();
        if temp < temp_2m {
            temp = temp_2m;
        }
        temp2 += temp_2m;
    }
    if temp2 > 20f64 {
        println!(
            "Added {} points for high average wind speed",
            (temp2 as i32) / 200
        );
        snowdaypoints += (temp2 as f32) / 200f32;
    }
    if temp > 20f64 {
        println!("Added 5 points for high wind speed");
        snowdaypoints += 5f32;
    }
    for i in 0..48 {
        if jsonval["hourly"]["precipitation_probability"][i]
            .as_f64()
            .unwrap()
            > 50f64
        {
            println!("Added 1 point for high precipitation probability");
            snowdaypoints += 1f32;
            break;
        }
    }
    temp = 10000f64;
    for i in 0..48 {
        if jsonval["hourly"]["apparent_temperature"][i]
            .as_f64()
            .unwrap()
            < temp
        {
            temp = jsonval["hourly"]["apparent_temperature"][i]
                .as_f64()
                .unwrap();
        }
    }
    if temp < 10f64 {
        println!("Added 2 points for low apparent temperature");
        snowdaypoints += 2f32;
    }
    snowdaypoints -= 15f32; // to  account for normal weather
    if snowdaypoints < 1f32 {
        snowdaypoints = 0.01f32;
    }
    if snowdaypoints > 99f32 {
        snowdaypoints = 99.99f32;
    }
    println!("{:?}% chance of snow day!", snowdaypoints);
    return Ok(snowdaypoints);
}
