{ system ? builtins.currentSystem }:
(import ./. { src = ./.; inherit system; }).shellNix.default
