# Bible Bot
A discord bot for quoting and referencing the bible.

### Usage  
**Single Verses** can be referenced at any time by just including the verse in this format: `Book Chapter:Verse` in any message.  
**Example:**  
![Single Verse Example](/screenshots/single_verse_example.png)  

**Multiple Verses** can also easily be included to any message using this format: `Book Chapter:VerseStart-VerseEnd` in any message.  
**Example:**  
![Multi-Verse Example](/screenshots/multi_verse_example.png)

---
### Commands
Bible Bot has only 2 commands:  
`/chapter [Book] [Chapter]` This will display an entire chapter. (This can be a lot of text so be careful using this!)  
**Example:**  
![Chapter Command Usage Example](/screenshots/chapter_cmd_usage.png)  
![Chapter Command Example](/screenshots/chapter_cmd.png)  

`/random_verse` This will display a random verse. This can be any verse in the bible so it may not make sense without the surrounding context.  
**Example:**  
![Random Verse Command Example](/screenshots/random_verse_cmd.png)  
`/reading_calc [Month (numeric)] [Day (numeric)] [year (optional)]` Shows what the daily reading will be for a specific date (3 chapters a day, 4 on weekends; read the entire bible in a year)
`/register_channel [option] [channel]` Register a channel to either be a daily verse channel or a reading schedule channel (see below)  
Options: `daily_verse` (to register a channel as a daily verse channel), `reading_schedule` (to register a channel as a reading schedule channel), and `remove` (to remove a registered channel).

### Registered Channels
There are two types of channel the bot can do; a daily verse channel and a reading schedule channel
