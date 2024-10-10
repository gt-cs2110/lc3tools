import { hideBin } from "yargs/helpers";

export async function cliEntrypoint(argv: string[]) {
    // HACK cause not ESM
    const yargs = (await import("yargs")).default;
    // HACK cause also not ESM
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const lc3: typeof import("lc3-backend") = require("lc3-backend");
    
    yargs(hideBin(argv))
        .command(["assemble <file>", "as"], "assemble into object files", yargs => {
            return yargs
                .positional("file", {
                    describe: "ASM file to assemble"
                })
            }, (argv) => {
                try {
                    lc3.assemble(argv.file as string);
                    console.log(lc3.getAndClearOutput());
                } catch (e) {
                    console.error(e.message);
                }
            })
        .command(["link <files..>", "ld"], "link object files", yargs => {
            return yargs
                .positional("files", {
                    describe: "Object files to link"
                })
                .option("o", {
                    alias: "output-file",
                    demandOption: true,
                    describe: "output for linked object file",
                    type: "string",
                    requiresArg: true
                })
            }, (argv) => {
                // TODO
                console.log(argv);
            })
        .command("$0", false, {}, (argv) => {
            console.log(`Invalid command ${argv._[0] ?? ''}`.trim());
        })
        .demandCommand()
        .help()
        .parse();

    process.exit();
}