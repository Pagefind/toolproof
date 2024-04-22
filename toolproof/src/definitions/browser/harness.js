const toolproof = new ToolproofHarness();

const inner = async () => {
  // insert_toolproof_inner_js
};

let inner_response;

try {
  inner_response = await inner();
} catch (e) {
  let errString = e.toString();
  if (/:toolproof_err:/.test(errString)) {
    toolproof.errors.push(errString.replace(/:toolproof_err: ?/, ""));
  } else {
    toolproof.errors.push(`JavaScript error: ${errString}`);
  }
}

if (toolproof.errors.length) {
  return {
    toolproof_errs: toolproof.errors,
    inner_response,
    logs: toolproof_log_events["ALL"].join("\n"),
  };
} else {
  return { toolproof_errs: [], inner_response };
}
