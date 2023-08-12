use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// View the difference between two file sizes
#[poise::command(prefix_command, slash_command)]
pub async fn file_details(
    ctx: Context<'_>,
    #[description = "File to examine"] file: serenity::Attachment,
    #[description = "Second file to examine"] file_2: Option<serenity::Attachment>,
) -> Result<(), Error> {
    ctx.say(format!(
        "First file name: **{}**. File size difference: **{}** bytes",
        file.filename,
        file.size - file_2.map_or(0, |f| f.size)
    ))
    .await?;
    Ok(())
}

#[poise::command(prefix_command)]
pub async fn totalsize(
    ctx: Context<'_>,
    #[description = "File to rename"] files: Vec<serenity::Attachment>,
) -> Result<(), Error> {
    let total = files.iter().map(|f| f.size as u64).sum::<u64>();

    ctx.say(format!(
        "Total file size: `{}B`. Average size: `{}B`",
        total,
        total.checked_div(files.len() as _).unwrap_or(0)
    ))
    .await?;

    Ok(())
}
