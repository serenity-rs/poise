(function() {
    var implementors = Object.fromEntries([["http",[]],["hyper",[]],["reqwest",[]]]);
    if (window.register_implementors) {
        window.register_implementors(implementors);
    } else {
        window.pending_implementors = implementors;
    }
})()
//{"start":57,"fragment_lengths":[11,13,15]}