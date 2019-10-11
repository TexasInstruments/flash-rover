// Copyright (c) 2019 , Texas Instruments.
// Licensed under the BSD-3-Clause license
// (see LICENSE or <https://opensource.org/licenses/BSD-3-Clause>) All files in the project
// notice may not be copied, modified, or distributed except according to those terms.

// Import the DSS packages into our namespace to save on typing
importPackage(Packages.com.ti.debug.engine.scripting)
importPackage(Packages.com.ti.ccstudio.scripting.environment)
importPackage(Packages.java.lang)
importPackage(Packages.java.io)

var SRAM_START      = 0x20000000;
var STACK_ADDR      = SRAM_START;
var RESET_ISR       = SRAM_START + 0x04;

var CONF            = 0x20003000;
var CONF_VALID      = CONF;
var CONF_MISO       = CONF + 0x04;
var CONF_MOSI       = CONF + 0x08;
var CONF_CLK        = CONF + 0x0C;
var CONF_CSN        = CONF + 0x10;

var DOORBELL        = 0x20003100;
var DOORBELL_CMD    = DOORBELL + 0x00;
var DOORBELL_RSP    = DOORBELL + 0x10;

var XFLASH_BUF      = 0x20004000;
var XFLASH_BUF_SIZE = 0x00001000;

var CMD_KIND = {
    xflashInfo:  0xC0,
    sectorErase: 0xC1,
    massErase:   0xC2,
    readBlock:   0xC3,
    writeBlock:  0xC4,
};

function Xflash(device) {
    this.cfg = {
        fw:      "dss/fw/" + device + ".bin",
        ccxml:   "dss/ccxml/" + device + ".ccxml",
        timeout: 15000, // 15 seconds
        log:     "dss/dss_log.xml",
        session: "Texas Instruments XDS110 USB Debug Probe/Cortex_M(3|4)_0",
    };

    this.script = ScriptingEnvironment.instance();

    this.script.traceBegin(this.cfg.log, "DefaultStylesheet.xsl");
    this.script.traceSetConsoleLevel(TraceLevel.OFF);
    this.script.traceSetFileLevel(TraceLevel.ALL);
    this.script.setScriptTimeout(this.cfg.timeout);

    this.debug_server = this.script.getServer("DebugServer.1");
    this.debug_server.setConfig(this.cfg.ccxml);

    this.debug_session = this.debug_server.openSession(this.cfg.session)
    this.debug_session.target.connect();
    this.debug_session.target.reset();
    this.debug_session.expression.evaluate("GEL_AdvancedReset(\"Board Reset (automatic connect/disconnect)\")");
}

Xflash.prototype.close = function() {
    this.debug_session.target.halt();
    this.debug_session.target.reset();
    this.debug_session.target.disconnect();
    this.debug_server.stop();

    this.script.traceSetConsoleLevel(TraceLevel.INFO)
    this.script.traceEnd();
}

Xflash.prototype.inject = function(conf) {
    this.debug_session.memory.loadRaw(0, SRAM_START, this.cfg.fw, 32, false);

    if (conf.valid) {
        this.debug_session.memory.writeData(0, CONF_VALID, 1, 32);
        this.debug_session.memory.writeData(0, CONF_MISO, conf.miso, 32);
        this.debug_session.memory.writeData(0, CONF_MOSI, conf.mosi, 32);
        this.debug_session.memory.writeData(0, CONF_CLK, conf.clk, 32);
        this.debug_session.memory.writeData(0, CONF_CSN, conf.csn, 32);
    }

    var stack_addr = this.debug_session.memory.readData(0, STACK_ADDR, 32, false);
    var reset_isr = this.debug_session.memory.readData(0, RESET_ISR, 32, false);

    this.debug_session.memory.writeRegister("MSP", stack_addr);
    this.debug_session.memory.writeRegister("PC", reset_isr);
    this.debug_session.memory.writeRegister("LR", 0xFFFFFFFF);

    this.debug_session.target.runAsynch();
}

Xflash.prototype.sendCommand = function(cmd) {
    var kind;

    this.debug_session.memory.writeData(0, DOORBELL_CMD + 0x0C, cmd.arg2, 32);
    this.debug_session.memory.writeData(0, DOORBELL_CMD + 0x08, cmd.arg1, 32);
    this.debug_session.memory.writeData(0, DOORBELL_CMD + 0x04, cmd.arg0, 32);
    // Kind must be written last to trigger the command
    this.debug_session.memory.writeData(0, DOORBELL_CMD + 0x00, cmd.kind, 32);

    // Wait for command to be consumed
    while (this.debug_session.memory.readData(0, DOORBELL_CMD, 32) != 0) {
        java.lang.Thread.sleep(100);
    }

    // Command consumed, wait for response
    while (this.debug_session.memory.readData(0, DOORBELL_RSP, 32) == 0) {
        java.lang.Thread.sleep(100);
    }

    var rsp = {
        kind: this.debug_session.memory.readData(0, DOORBELL_RSP + 0x00, 32),
        arg0: this.debug_session.memory.readData(0, DOORBELL_RSP + 0x04, 32),
        arg1: this.debug_session.memory.readData(0, DOORBELL_RSP + 0x08, 32),
        arg2: this.debug_session.memory.readData(0, DOORBELL_RSP + 0x0C, 32),
    };

    // Consume response
    this.debug_session.memory.writeData(0, DOORBELL_RSP, 0x00, 32);

    return rsp;
}

Xflash.prototype.info = function() {
    var xflashInfoCmd = {
        kind: CMD_KIND.xflashInfo,
        arg0: 0x00,
        arg1: 0x00,
        arg2: 0x00,
    };

    var rsp = this.sendCommand(xflashInfoCmd);
    if (rsp.kind != 0xD1) {
        throw "Xflash Error: 0x" + rsp.kind.toString(16).toUpperCase();
    }

    print(rsp.arg0.toString() + " " + rsp.arg1.toString()
    );
}

Xflash.prototype.sectorErase = function(offset, length) {
    var sectorEraseCmd = {
        kind: CMD_KIND.sectorErase,
        arg0: offset,
        arg1: length,
        arg2: 0x00,
    };

    var rsp = this.sendCommand(sectorEraseCmd);
    if (rsp.kind != 0xD0) {
        throw "Xflash Error: 0x" + rsp.kind.toString(16).toUpperCase();
    }
}

Xflash.prototype.massErase = function() {
    var massEraseCmd = {
        kind: CMD_KIND.massErase,
        arg0: 0x00,
        arg1: 0x00,
        arg2: 0x00,
    };

    var rsp = this.sendCommand(massEraseCmd);
    if (rsp.kind != 0xD0) {
        throw "Xflash Error: 0x" + rsp.kind.toString(16).toUpperCase();
    }
}

Xflash.prototype.read = function(offset, length) {
    while (length > 0) {
        var ilength = java.lang.Math.min(length, XFLASH_BUF_SIZE);

        var readBlockCmd = {
            kind: CMD_KIND.readBlock,
            arg0: offset,
            arg1: ilength,
            arg2: 0x00,
        };

        var rsp = this.sendCommand(readBlockCmd);
        if (rsp.kind != 0xD0) {
            throw "Xflash Error: 0x" + rsp.kind.toString(16).toUpperCase();
        }

        // This is a sloppy implementation because the readData() API returns
        // a long[] array, while the write() API only accepts byte[] arrays. I
        // haven't found a trivial conversion between these two arrays without
        // manually writing byte by byte. I am sorry.
        var data = this.debug_session.memory.readData(0, XFLASH_BUF, 8, ilength);
        for (var i = 0; i < ilength; i++) {
            System.out.write(data[i]);
        }

        length -= ilength;
        offset += ilength;
    }

    System.out.flush();
    System.out.close();
}

Xflash.prototype.write_fixed = function(offset, length, erase) {
    var is = System['in'];
    var is_buf = java.lang.reflect.Array.newInstance(java.lang.Byte.TYPE, XFLASH_BUF_SIZE);

    if (erase) {
        this.sectorErase(offset, length);
    }

    while (length > 0) {
        var ilength = java.lang.Math.min(length, XFLASH_BUF_SIZE);
        ilength = is.read(is_buf, 0, ilength);

        // This is a sloppy implementation because the writeData() API takes a
        // long[] array, while the read() API only accepts byte[] arrays. I
        // haven't found a trivial conversion between these two arrays without
        // manually reading byte by byte. I am sorry.
        var buf = new Array();
        for (var i = 0; i < ilength; i++) {
            buf.push(is_buf[i]);
        }

        this.debug_session.memory.writeData(0, XFLASH_BUF, buf, 8);

        var writeBlockCmd = {
            kind: CMD_KIND.writeBlock,
            arg0: offset,
            arg1: ilength,
            arg2: 0x00,
        };

        var rsp = this.sendCommand(writeBlockCmd);
        if (rsp.kind != 0xD0) {
            throw "Xflash Error: 0x" + rsp.kind.toString(16).toUpperCase();
        }

        length -= ilength;
        offset += ilength;
    }
}

Xflash.prototype.write_stream = function(offset, erase) {
    var is = System['in'];
    var is_buf = java.lang.reflect.Array.newInstance(java.lang.Byte.TYPE, XFLASH_BUF_SIZE);

    var erase_boundary = offset;

    while (is.available() > 0) {
        var ilength = is.read(is_buf);

        // This is a sloppy implementation because the writeData() API takes a
        // long[] array, while the read() API only accepts byte[] arrays. I
        // haven't found a trivial conversion between these two arrays without
        // manually reading byte by byte. I am sorry.
        var buf = new Array();
        for (var i = 0; i < ilength; i++) {
            buf.push(is_buf[i]);
        }

        if (erase) {
            var iend = offset + ilength - 1;
            if (iend >= erase_boundary) {
                this.sectorErase(offset, ilength);
                var sector_size = 4096;
                erase_boundary = Math.floor((iend + sector_size) / sector_size) * sector_size;                
            }
        }
        
        this.debug_session.memory.writeData(0, XFLASH_BUF, buf, 8);

        var writeBlockCmd = {
            kind: CMD_KIND.writeBlock,
            arg0: offset,
            arg1: ilength,
            arg2: 0x00,
        };

        var rsp = this.sendCommand(writeBlockCmd);
        if (rsp.kind != 0xD0) {
            throw "Xflash Error: 0x" + rsp.kind.toString(16).toUpperCase();
        }

        offset += ilength;
    }
}

if (arguments.length == 0) {
    java.lang.System.exit(0);
}

var i = 0;

var device = arguments[i++];

var conf = {
    valid: 0,
    miso: 0xFFFFFFFF,
    mosi: 0xFFFFFFFF,
    clk: 0xFFFFFFFF,
    csn: 0xFFFFFFFF,
};

if (arguments[i] == "conf") {
    i++;
    conf.valid = 1;
    conf.miso = parseInt(arguments[i++]);
    conf.mosi = parseInt(arguments[i++]);
    conf.clk = parseInt(arguments[i++]);
    conf.csn = parseInt(arguments[i++]);
}

try {
    var xflash = new Xflash(device);
    xflash.inject(conf);

    var command = arguments[i++];
    switch (command) {
        case "info":
            xflash.info();
            break;

        case "mass-erase":
            xflash.massErase();
            break;

        case "sector-erase":
            var offset = parseInt(arguments[i++]);
            var length = parseInt(arguments[i++]);

            xflash.sectorErase(offset, length);
            break;

        case "read":
            var offset = parseInt(arguments[i++]);
            var length = parseInt(arguments[i++]);

            xflash.read(offset, length);
            break;

        case "write":
            var offset = parseInt(arguments[i++]);
            var length = parseInt(arguments[i++]);
            var erase = !!(parseInt(arguments[i++]) || false);

            if (length != -1) {
                xflash.write_fixed(offset, length, erase);
            } else {
                xflash.write_stream(offset, erase);
            }
            break;

        default:
            throw "Invalid command: " + command;
    }
} catch (ex) {
    if (xflash !== undefined) {
        xflash.close();
    }
    System.err.write(ex);
    java.lang.System.exit(1);
} 

xflash.close();
java.lang.System.exit(0);
