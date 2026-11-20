-- Card searches run signed out on zwipe.net, kept as one running count that the public searches
-- total adds in. One row and no row per search: no account, address, query, session or time is
-- stored, only how many. A new table, so no lock falls on a live one. zerver owns and writes it;
-- zervice reads nothing here.
CREATE TABLE anonymous_searches (
    id       BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    searches BIGINT  NOT NULL DEFAULT 0
);

INSERT INTO anonymous_searches DEFAULT VALUES;
