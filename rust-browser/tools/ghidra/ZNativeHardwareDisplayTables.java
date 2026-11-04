//Numeric data tables and bounded UNIT98 writer inventory; no asset pixels.
import ghidra.app.script.GhidraScript;import ghidra.program.model.listing.*;import java.io.PrintWriter;
public class ZNativeHardwareDisplayTables extends GhidraScript{public void run()throws Exception{try(PrintWriter o=new PrintWriter(getScriptArgs()[0])){
 long[]tables={0x496dfc,0x496ba4,0x496ca4,0x4d1900,0x496d24,0x496c14,0x496ce4,0x496ddc,0x496d04,0x496c24,0x496b84,0x496c44,0x496bc4,0x496d44,0x496b44,0x496b24,0x496b64};
 for(long a:tables){int n=a==0x496dfc||a==0x4d1900||a==0x496c14?4:8;o.print(Long.toHexString(a));for(int i=0;i<n;i++)o.print(","+currentProgram.getMemory().getShort(toAddr(a+i*4))+","+currentProgram.getMemory().getShort(toAddr(a+i*4+2)));o.println();}
 o.println("WRITERS98");InstructionIterator it=currentProgram.getListing().getInstructions(true);while(it.hasNext()){Instruction i=it.next();if(i.getNumOperands()==0)continue;String p=i.getDefaultOperandRepresentation(0),m=i.getMnemonicString();if(p.contains(" + 0x98]")&&!m.equals("CMP")&&!m.equals("TEST")&&!m.equals("CALL")&&!m.equals("PUSH")){Function f=currentProgram.getFunctionManager().getFunctionContaining(i.getAddress());o.println(i.getAddress()+" "+i+" FUNCTION "+(f==null?"null":f.getEntryPoint()));}}
}}}
