-- The shared database for tran4vert, for a Supabase project the person creates.
-- Paste this whole file into the project's SQL editor and run it once. It holds
-- no secret: passcodes are set afterwards, by the person, in the SQL editor.
--
-- The page ships in a public repository with a *publishable* key, so every table
-- is closed to that key. The page reaches data only through the functions below.
--
-- Signing in: an athlete taps their name, then the vertical they want (their
-- "goal", in inches, set by the coach). The page sends it as "<athlete id>:<goal>".
-- That is a light lock, not a strong one: there are only a few choices, so five
-- wrong tries for one athlete lock that athlete out for fifteen minutes
-- (`too_many`), and every wrong try waits half a second (`bad_passcode`). The
-- coach has a typed passcode, which also opens everything. The coach functions
-- need it (`coach_only` otherwise). An athlete changes only their own maxes and
-- moves (`not_yours`). Audio is capped at 2 MB.
--
-- Safe to run again: it keeps the data and replaces the functions.
--
-- First run, in the same editor (type your own coach passcode):
--
--   select set_passcode('coach', 'the coach passcode');
--   insert into athletes (name, start, goal) values ('First name', '2026-11-30', 40);  -- once per athlete; a Monday; the goal they will tap
--
-- then run `python3.13 tools/seed.py` on your computer to load the program.

create extension if not exists pgcrypto with schema extensions;
create schema if not exists private;

-- ---- tables, all closed ----

create table if not exists public.program (
  id int primary key default 1 check (id = 1),
  body jsonb not null
);

create table if not exists public.athletes (
  id int generated always as identity primary key,
  name text not null check (char_length(name) between 1 and 40),
  start date not null check (extract(isodow from start) = 1),
  goal int check (goal between 10 and 99)
);
alter table public.athletes add column if not exists goal int check (goal between 10 and 99);
-- A picture the athlete or the coach uploads (at most 300 KB). It is served to anyone with the link.
alter table public.athletes add column if not exists avatar bytea check (octet_length(avatar) <= 307200);
alter table public.athletes add column if not exists avatar_mime text check (avatar_mime like 'image/%' and char_length(avatar_mime) <= 60);

create table if not exists public.attempts (
  athlete int not null,
  at timestamptz not null default now()
);

create table if not exists public.maxes (
  id bigint generated always as identity primary key,
  athlete int not null references public.athletes (id) on delete cascade,
  lift text not null check (char_length(lift) between 1 and 60),
  weight int not null check (weight between 1 and 1500),
  date date not null
);

create table if not exists public.moves (
  athlete int not null references public.athletes (id) on delete cascade,
  cycle text not null,
  week int not null check (week between 1 and 4),
  dow int not null check (dow between 0 and 6),
  to_date date not null,
  primary key (athlete, cycle, week, dow)
);

create table if not exists public.voice_notes (
  id bigint generated always as identity primary key,
  lift text not null check (char_length(lift) between 1 and 60),
  title text not null check (char_length(title) between 1 and 60),
  seconds int not null check (seconds between 1 and 180),
  date date not null,
  mime text not null check (mime like 'audio/%' and char_length(mime) <= 60),
  audio bytea not null check (octet_length(audio) between 1 and 2097152)
);

create table if not exists public.cycle_notes (
  cycle text primary key,
  text text not null check (char_length(text) between 1 and 500)
);

create table if not exists public.secrets (
  name text primary key check (name in ('group', 'coach')),
  hash text not null
);

alter table public.program enable row level security;
alter table public.athletes enable row level security;
alter table public.maxes enable row level security;
alter table public.moves enable row level security;
alter table public.voice_notes enable row level security;
alter table public.cycle_notes enable row level security;
alter table public.secrets enable row level security;
alter table public.attempts enable row level security;
-- No policies on purpose: with row-level security on and no policy, the
-- publishable key can read and write nothing in these tables directly.
revoke all on all tables in schema public from anon, authenticated;
revoke all on all sequences in schema public from anon, authenticated;

-- ---- the passcode check, not callable from outside ----

create or replace function private.role_of(p_code text)
returns text language plpgsql security definer set search_path = public, extensions as $$
declare c text; a int; v_goal int; failed int;
begin
  if p_code ~ '^[0-9]{1,9}:[0-9]{1,3}$' then
    a := split_part(p_code, ':', 1)::int;
    v_goal := split_part(p_code, ':', 2)::int;
    select count(*) into failed from public.attempts where athlete = a and at > now() - interval '15 minutes';
    if failed >= 5 then
      perform pg_sleep(0.5);
      raise exception 'too_many';
    end if;
    if exists (select 1 from public.athletes x where x.id = a and x.goal is not null and x.goal = v_goal) then
      return 'athlete';
    end if;
    insert into public.attempts (athlete) values (a);
    delete from public.attempts where at < now() - interval '1 day';
    perform pg_sleep(0.5);
    raise exception 'bad_passcode';
  end if;
  select hash into c from public.secrets where name = 'coach';
  if p_code is not null and c is not null and c = crypt(p_code, c) then return 'coach'; end if;
  perform pg_sleep(0.5);
  raise exception 'bad_passcode';
end $$;

-- The athlete a sign-in belongs to, or null for the coach.
create or replace function private.athlete_of(p_code text)
returns int language sql immutable as $$
  select case when p_code ~ '^[0-9]{1,9}:[0-9]{1,3}$' then split_part(p_code, ':', 1)::int end
$$;

-- An athlete may change their own data; the coach may change anyone's.
create or replace function private.can_act(p_code text, p_athlete int)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  if private.role_of(p_code) = 'coach' then return; end if;
  if private.athlete_of(p_code) is distinct from p_athlete then raise exception 'not_yours'; end if;
end $$;

create or replace function private.need_coach(p_code text)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  if private.role_of(p_code) <> 'coach' then
    raise exception 'coach_only';
  end if;
end $$;

-- A date given as text, or a clear error.
create or replace function private.as_date(p_text text)
returns date language plpgsql immutable as $$
begin
  return p_text::date;
exception when others then
  raise exception 'bad_date';
end $$;

-- ---- setting a passcode: run by the person in the SQL editor, never by the page ----

create or replace function public.set_passcode(p_kind text, p_code text)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  if p_kind <> 'coach' then raise exception 'only the coach has a passcode'; end if;
  if p_code is null or char_length(p_code) < 4 then raise exception 'use at least 4 characters'; end if;
  insert into public.secrets (name, hash) values (p_kind, crypt(p_code, gen_salt('bf')))
  on conflict (name) do update set hash = excluded.hash;
end $$;

-- ---- reading ----

create or replace function public.get_state(p_code text)
returns jsonb language plpgsql security definer set search_path = public, extensions as $$
declare r text := private.role_of(p_code);
begin
  return jsonb_build_object(
    'role', r,
    'me', private.athlete_of(p_code),
    'as_of', to_char(current_date, 'YYYY-MM-DD'),
    'program', coalesce((select body from public.program where id = 1), '{}'::jsonb),
    'athletes', coalesce((select jsonb_agg(jsonb_build_object('id', id, 'name', name,
        'start', to_char(start, 'YYYY-MM-DD'), 'avatar', avatar is not null) || case when r = 'coach' and goal is not null
        then jsonb_build_object('goal', goal) else '{}'::jsonb end order by id) from public.athletes), '[]'::jsonb),
    'maxes', coalesce((select jsonb_agg(jsonb_build_object('id', id, 'athlete', athlete, 'lift', lift,
        'weight', weight, 'date', to_char(date, 'YYYY-MM-DD')) order by id) from public.maxes), '[]'::jsonb),
    'moves', coalesce((select jsonb_agg(jsonb_build_object('athlete', athlete, 'cycle', cycle, 'week', week,
        'dow', dow, 'to', to_char(to_date, 'YYYY-MM-DD')) order by athlete, cycle, week, dow) from public.moves), '[]'::jsonb),
    'notes', coalesce((select jsonb_agg(jsonb_build_object('id', id, 'lift', lift, 'title', title,
        'seconds', seconds, 'date', to_char(date, 'YYYY-MM-DD')) order by id) from public.voice_notes), '[]'::jsonb),
    'cycle_notes', coalesce((select jsonb_agg(jsonb_build_object('cycle', cycle, 'text', text) order by cycle)
        from public.cycle_notes), '[]'::jsonb)
  );
end $$;

-- The names to tap, before anyone is signed in. Names and ids only.
create or replace function public.get_names()
returns jsonb language sql security definer set search_path = public, extensions as $$
  select coalesce(jsonb_agg(jsonb_build_object('id', id, 'name', name, 'avatar', avatar is not null) order by id), '[]'::jsonb)
  from public.athletes
$$;

-- An athlete's picture, to anyone: it shows before anyone is signed in.
create or replace function public.get_avatar(p_athlete int)
returns bytea language plpgsql security definer set search_path = public, extensions as $$
declare b bytea;
begin
  select avatar into b from public.athletes where id = p_athlete;
  if b is null then raise exception 'no_such_avatar'; end if;
  return b;
end $$;

create or replace function public.set_avatar(p_code text, p_athlete int, p_mime text, p_image text)
returns void language plpgsql security definer set search_path = public, extensions as $$
declare bytes bytea;
begin
  perform private.can_act(p_code, p_athlete);
  begin
    bytes := decode(p_image, 'base64');
  exception when others then
    raise exception 'bad_image';
  end;
  if octet_length(bytes) > 307200 then raise exception 'image_too_large'; end if;
  update public.athletes set avatar = bytes, avatar_mime = p_mime where id = p_athlete;
  if not found then raise exception 'no_such_athlete'; end if;
end $$;

create or replace function public.get_note_audio(p_code text, p_id bigint)
returns bytea language plpgsql security definer set search_path = public, extensions as $$
declare b bytea;
begin
  perform private.role_of(p_code);
  select audio into b from public.voice_notes where id = p_id;
  if b is null then raise exception 'no_such_note'; end if;
  return b;
end $$;

-- ---- changes an athlete may make ----

create or replace function public.add_max(p_code text, p_athlete int, p_lift text, p_weight int, p_date text)
returns bigint language plpgsql security definer set search_path = public, extensions as $$
declare new_id bigint;
begin
  perform private.can_act(p_code, p_athlete);
  if not exists (select 1 from public.athletes where id = p_athlete) then raise exception 'no_such_athlete'; end if;
  if not coalesce((select (body -> 'lifts') ? p_lift from public.program where id = 1), false) then
    raise exception 'no_such_lift';
  end if;
  insert into public.maxes (athlete, lift, weight, date)
  values (p_athlete, p_lift, p_weight, private.as_date(p_date)) returning id into new_id;
  return new_id;
end $$;

create or replace function public.delete_max(p_code text, p_id bigint)
returns void language plpgsql security definer set search_path = public, extensions as $$
declare a int;
begin
  select athlete into a from public.maxes where id = p_id;
  perform private.can_act(p_code, a);
  delete from public.maxes where id = p_id;
end $$;

create or replace function public.move_session(p_code text, p_athlete int, p_cycle text, p_week int, p_dow int, p_to text)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  perform private.can_act(p_code, p_athlete);
  if not exists (select 1 from public.athletes where id = p_athlete) then raise exception 'no_such_athlete'; end if;
  if not exists (select 1 from public.program, jsonb_array_elements(body -> 'cycles') c
                 where id = 1 and c ->> 'id' = p_cycle) then raise exception 'no_such_session'; end if;
  insert into public.moves (athlete, cycle, week, dow, to_date)
  values (p_athlete, p_cycle, p_week, p_dow, private.as_date(p_to))
  on conflict (athlete, cycle, week, dow) do update set to_date = excluded.to_date;
end $$;

create or replace function public.undo_move(p_code text, p_athlete int, p_cycle text, p_week int, p_dow int)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  perform private.can_act(p_code, p_athlete);
  delete from public.moves where athlete = p_athlete and cycle = p_cycle and week = p_week and dow = p_dow;
end $$;

-- ---- changes only the coach may make ----

create or replace function public.add_note(p_code text, p_lift text, p_title text, p_seconds int, p_date text, p_mime text, p_audio text)
returns bigint language plpgsql security definer set search_path = public, extensions as $$
declare new_id bigint; bytes bytea;
begin
  perform private.need_coach(p_code);
  if not coalesce((select (body -> 'lifts') ? p_lift from public.program where id = 1), false) then
    raise exception 'no_such_lift';
  end if;
  begin
    bytes := decode(p_audio, 'base64');
  exception when others then
    raise exception 'bad_audio';
  end;
  if octet_length(bytes) > 2097152 then raise exception 'audio_too_large'; end if;
  insert into public.voice_notes (lift, title, seconds, date, mime, audio)
  values (p_lift, btrim(p_title), p_seconds, private.as_date(p_date), p_mime, bytes) returning id into new_id;
  return new_id;
end $$;

create or replace function public.delete_note(p_code text, p_id bigint)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  perform private.need_coach(p_code);
  delete from public.voice_notes where id = p_id;
end $$;

create or replace function public.set_start_date(p_code text, p_athlete int, p_start text)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  perform private.need_coach(p_code);
  update public.athletes set start = private.as_date(p_start) where id = p_athlete;
  if not found then raise exception 'no_such_athlete'; end if;
end $$;

create or replace function public.set_goal(p_code text, p_athlete int, p_goal int)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  perform private.need_coach(p_code);
  update public.athletes set goal = p_goal where id = p_athlete;
  if not found then raise exception 'no_such_athlete'; end if;
end $$;

create or replace function public.set_cycle_note(p_code text, p_cycle text, p_text text)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  perform private.need_coach(p_code);
  if btrim(p_text) = '' then
    delete from public.cycle_notes where cycle = p_cycle;
  else
    insert into public.cycle_notes (cycle, text) values (p_cycle, btrim(p_text))
    on conflict (cycle) do update set text = excluded.text;
  end if;
end $$;

-- The program as one document. The private maxes the PDFs held are dropped here
-- as well as by the seed tool, so a friend's phone never receives them.
create or replace function public.set_program(p_code text, p_body jsonb)
returns void language plpgsql security definer set search_path = public, extensions as $$
begin
  perform private.need_coach(p_code);
  insert into public.program (id, body) values (1, p_body - 'maxes_from_pdf')
  on conflict (id) do update set body = excluded.body;
end $$;

-- ---- who may call what: the publishable key calls the functions, and nothing else ----

revoke all on schema private from public, anon, authenticated;
revoke execute on all functions in schema public from public, anon, authenticated;
grant execute on function
  public.get_names(), public.get_avatar(int), public.set_avatar(text, int, text, text), public.set_goal(text, int, int),
  public.get_state(text), public.get_note_audio(text, bigint),
  public.add_max(text, int, text, int, text), public.delete_max(text, bigint),
  public.move_session(text, int, text, int, int, text), public.undo_move(text, int, text, int, int),
  public.add_note(text, text, text, int, text, text, text), public.delete_note(text, bigint),
  public.set_start_date(text, int, text), public.set_cycle_note(text, text, text),
  public.set_program(text, jsonb)
to anon, authenticated;
-- set_passcode is not granted to anyone: only the SQL editor (the project owner) runs it.
