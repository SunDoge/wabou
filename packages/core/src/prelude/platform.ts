// Native parsing, small JS wrappers for Web API identity and iteration semantics.
declare function __wabou_url(
  input: string,
  base: string,
  field: string,
  value: string,
): Record<string, string>;
declare function __wabou_form_decode(input: string): string;
declare function __wabou_form_encode(input: string): string;

// Web IDL USVString replaces lone surrogates before crossing the UTF-8 boundary.
function usv(value: unknown): string {
  return String(value).replace(
    /[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/g,
    "\ufffd",
  );
}
class NativeURLSearchParams {
  private pairs: [string, string][];
  private update?: (query: string) => void;
  constructor(
    init: string | Iterable<Iterable<string>> | Record<string, string> = "",
  ) {
    if (typeof init === "string")
      this.pairs = JSON.parse(__wabou_form_decode(init.replace(/^\?/, "")));
    else if (Symbol.iterator in Object(init)) {
      this.pairs = Array.from(init as Iterable<Iterable<string>>, (pair) => {
        const values = Array.from(pair);
        if (values.length !== 2)
          throw new TypeError("Each query pair must contain two values");
        return [usv(values[0]), usv(values[1])];
      });
    } else this.pairs = Object.entries(init).map(([k, v]) => [usv(k), usv(v)]);
  }
  // Internal hooks preserve the same searchParams object across URL mutations.
  connect(update: (query: string) => void) {
    this.update = update;
  }
  replace(query: string) {
    this.pairs = JSON.parse(__wabou_form_decode(query.replace(/^\?/, "")));
  }
  private changed() {
    this.update?.(this.toString());
  }
  get size() {
    return this.pairs.length;
  }
  append(name: string, value: string) {
    this.pairs.push([usv(name), usv(value)]);
    this.changed();
  }
  delete(name: string, value?: string) {
    name = usv(name);
    if (value !== undefined) value = usv(value);
    this.pairs = this.pairs.filter(
      ([k, v]) => k !== name || (value !== undefined && v !== value),
    );
    this.changed();
  }
  get(name: string) {
    return this.pairs.find(([k]) => k === usv(name))?.[1] ?? null;
  }
  getAll(name: string) {
    return this.pairs.filter(([k]) => k === usv(name)).map(([, v]) => v);
  }
  has(name: string, value?: string) {
    return this.pairs.some(
      ([k, v]) => k === usv(name) && (value === undefined || v === usv(value)),
    );
  }
  set(name: string, value: string) {
    name = usv(name);
    value = usv(value);
    const first = this.pairs.findIndex(([k]) => k === name);
    if (first < 0) this.pairs.push([name, value]);
    else {
      this.pairs[first][1] = value;
      this.pairs = this.pairs.filter(([k], i) => k !== name || i === first);
    }
    this.changed();
  }
  sort() {
    this.pairs.sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
    this.changed();
  }
  *entries(): IterableIterator<[string, string]> {
    for (let i = 0; i < this.pairs.length; i++) yield [...this.pairs[i]];
  }
  *keys() {
    for (const [k] of this.entries()) yield k;
  }
  *values() {
    for (const [, v] of this.entries()) yield v;
  }
  [Symbol.iterator]() {
    return this.entries();
  }
  forEach(
    callback: (
      value: string,
      key: string,
      parent: NativeURLSearchParams,
    ) => void,
    thisArg?: unknown,
  ) {
    for (const [k, v] of this.entries()) callback.call(thisArg, v, k, this);
  }
  toString() {
    return __wabou_form_encode(JSON.stringify(this.pairs));
  }
  get [Symbol.toStringTag]() {
    return "URLSearchParams";
  }
}
class NativeURL {
  private parts: Record<string, string>;
  readonly searchParams: NativeURLSearchParams;
  constructor(input: string, base?: string) {
    if (base !== undefined && usv(base) === "")
      throw new TypeError("Invalid base URL");
    this.parts = __wabou_url(
      usv(input),
      base === undefined ? "" : usv(base),
      "",
      "",
    );
    this.searchParams = new NativeURLSearchParams(this.parts.search);
    this.searchParams.connect((query) => this.setPart("search", query));
  }
  private setPart(name: string, value: string) {
    this.parts = __wabou_url(this.parts.href, "", name, usv(value));
    if (name === "href" || name === "search")
      this.searchParams.replace(this.parts.search);
  }
  get href() {
    return this.parts.href;
  }
  set href(v: string) {
    this.setPart("href", v);
  }
  get origin() {
    return this.parts.origin;
  }
  get protocol() {
    return this.parts.protocol;
  }
  set protocol(v: string) {
    this.setPart("protocol", v);
  }
  get username() {
    return this.parts.username;
  }
  set username(v: string) {
    this.setPart("username", v);
  }
  get password() {
    return this.parts.password;
  }
  set password(v: string) {
    this.setPart("password", v);
  }
  get host() {
    return this.parts.host;
  }
  set host(v: string) {
    this.setPart("host", v);
  }
  get hostname() {
    return this.parts.hostname;
  }
  set hostname(v: string) {
    this.setPart("hostname", v);
  }
  get port() {
    return this.parts.port;
  }
  set port(v: string) {
    this.setPart("port", v);
  }
  get pathname() {
    return this.parts.pathname;
  }
  set pathname(v: string) {
    this.setPart("pathname", v);
  }
  get search() {
    return this.parts.search;
  }
  set search(v: string) {
    this.setPart("search", v);
  }
  get hash() {
    return this.parts.hash;
  }
  set hash(v: string) {
    this.setPart("hash", v);
  }
  toString() {
    return this.href;
  }
  toJSON() {
    return this.href;
  }
  static canParse(input: string, base?: string) {
    return this.parse(input, base) !== null;
  }
  static parse(input: string, base?: string) {
    try {
      return new NativeURL(input, base);
    } catch {
      return null;
    }
  }
  get [Symbol.toStringTag]() {
    return "URL";
  }
}
const runtime = globalThis as Record<string, any>;
runtime.URL ??= NativeURL;
runtime.URLSearchParams ??= NativeURLSearchParams;
