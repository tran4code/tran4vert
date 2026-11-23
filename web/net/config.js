// The one file that names another origin (R7, a declared off-origin feature).
// Both values come from the person's own Supabase project: its address and its
// publishable (anon) key. The key ships in a public page, so the database is
// closed to it except through functions that check the passcode (supabase/schema.sql).
// Left empty, the page runs as a demo with made-up data and makes no request.
export const config = {
  url: "https://eqkromjfmsarwtzomtvs.supabase.co",
  key: "sb_publishable_PgfKMvcKgDHDo71AV76BIA_wYsGDfvn",
};
