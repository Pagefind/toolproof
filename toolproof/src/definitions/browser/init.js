let toolproof_log_events = {
  ALL: [],
  LOG: [],
  WRN: [],
  ERR: [],
  DBG: [],
};

(function () {
  const c = console;
  c.events = [];
  let l = [c.log, c.warn, c.error, c.debug].map((e) => e.bind(c));
  let p = (m, a) => {
    toolproof_log_events["ALL"].push(`${m}: ${Array.from(a).join(" ")}`);
    toolproof_log_events[m].push(`${Array.from(a).join(" ")}`);
  };
  c.log = function () {
    l[0].apply(c, arguments);
    p("LOG", arguments);
  };
  c.warn = function () {
    l[1].apply(c, arguments);
    p("WRN", arguments);
  };
  c.error = function () {
    l[2].apply(c, arguments);
    p("ERR", arguments);
  };
  c.debug = function () {
    l[3].apply(c, arguments);
    p("DBG", arguments);
  };
})();

class ToolproofHarness {
  constructor() {
    this.errors = [];
  }

  assert_eq(left, right) {
    if (left !== right) {
      this.errors.push(
        `Equality Assertion failed. Left: ${JSON.stringify(
          left
        )}, Right: ${JSON.stringify(right)}`
      );
    }
  }

  assert_lte(left, right) {
    if (left > right) {
      this.errors.push(
        `<= Assertion failed. Left: ${JSON.stringify(
          left
        )}, was greater than Right: ${JSON.stringify(right)}`
      );
    }
  }

  assert_gte(left, right) {
    if (left < right) {
      this.errors.push(
        `>= Assertion failed. Left: ${JSON.stringify(
          left
        )}, was less than Right: ${JSON.stringify(right)}`
      );
    }
  }

  assert(value) {
    if (!value) {
      this.errors.push(
        `Assertion failed, value was falsey. Value: ${JSON.stringify(value)}`
      );
    }
  }

  async waitFor(q, timeout = DEFAULT_TIMEOUT) {
    let start = Date.now();
    const throttle = 50; // TODO: configure

    let r = await q();
    while (!r) {
      await new Promise((r) => setTimeout(r, throttle));
      r = await q();
      if (Date.now() - start > timeout) {
        break;
      }
    }
    if (!r) {
      throw new Error(
        `:toolproof_err: waitFor timed out at ${timeout}ms, no result for "${q.toString()}"`
      );
    }
    return r;
  }

  async querySelector(s, timeout = DEFAULT_TIMEOUT) {
    try {
      return await this.waitFor(() => document.querySelector(s), timeout);
    } catch (e) {
      if (/:toolproof_err:/.test(e.toString())) {
        throw new Error(
          `:toolproof_err: querySelector timed out at ${timeout}ms, no elements matching "${s}"`
        );
      } else {
        throw e;
      }
    }
  }

  async querySelectorAll(s, timeout = DEFAULT_TIMEOUT) {
    try {
      return await this.waitFor(() => {
        let els = document.querySelectorAll(s);
        if (!els?.length) return null;
        return els;
      }, timeout);
    } catch (e) {
      if (/:toolproof_err:/.test(e.toString())) {
        throw new Error(
          `:toolproof_err: querySelectorAll timed out at ${timeout}ms, no elements matching "${s}"`
        );
      } else {
        throw e;
      }
    }
  }
}
