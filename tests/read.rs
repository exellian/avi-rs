#[cfg(test)]
mod tests {
    use tokio::fs::File;
    use avi_rs::AviAsyncReader;
    use std::error::Error;

    #[tokio::test]
    async fn parse_header() -> Result<(), Box<dyn Error>> {
        let file = File::open("tests/raw_sound.avi").await?;

        let reader = AviAsyncReader::read_header(file).await?;

        println!("Res: {:?}", reader);

        Ok(())
    }
}

