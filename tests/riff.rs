#[cfg(test)]
mod tests {
    use tokio::fs::File;
    use std::error::Error;
    use avi_rs::riff::RiffTree;

    #[tokio::test]
    async fn parse_async() -> Result<(), Box<dyn Error>> {
        let mut file = File::open("tests/raw_sound.avi").await?;

        let tree = RiffTree::read_async(&mut file).await?;

        println!("Riff Tree childs: {}", tree.childs().len());
        println!("Riff Tree: {:#?}", tree);

        Ok(())
    }

    #[test]
    fn parse() -> Result<(), Box<dyn Error>> {
        let mut file = std::fs::File::open("tests/raw_sound.avi")?;

        let tree = RiffTree::read(&mut file)?;

        println!("Riff Tree: {:?}", tree);

        Ok(())
    }
}

