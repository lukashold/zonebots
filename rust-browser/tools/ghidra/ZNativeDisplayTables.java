// Numeric drawing-table facts; no pixel resources or decompilation exported.
import ghidra.app.script.GhidraScript;
import java.io.PrintWriter;
public class ZNativeDisplayTables extends GhidraScript{public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){out.print("depth=[");for(int i=0;i<78;i++)out.print((i==0?"":",")+currentProgram.getMemory().getByte(toAddr(0x498f38+i*26+19)));out.println("]");out.print("phase=[");for(int i=0;i<76;i++)out.print((i==0?"":",")+"["+currentProgram.getMemory().getShort(toAddr(0x496e78+i*4))+","+currentProgram.getMemory().getShort(toAddr(0x496e7a+i*4))+"]");out.println("]");}}}
