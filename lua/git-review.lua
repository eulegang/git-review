--- @meta

--- @class Review
--- @field theme fun(theme: Theme): nil
--- @field treesitter Treesitter
--- @field diff Keys
--- @field selector Keys
--- @field filter Keys

--- @class Treesitter
--- @field path fun(path: string[])
--- @field extensions fun(ext: { [string]: string })
--- @field highlights fun(ext: { [string]: string })

--- @class Theme
--- @field added_bg? string
--- @field selected_added_bg? string
--- @field removed_bg? string
--- @field selected_removed_bg? string
--- @field binary_bg? string
--- @field selector_highlight_fg? string
--- @field warning_fg? string
--- @field hunk_header_fg? string

--- @class Keys
--- @field bind fun(record: { [string]: string })
--- @field unbind fun(pattern: string | string[])
--- @field clear fun()


--- @type Review
review = review
