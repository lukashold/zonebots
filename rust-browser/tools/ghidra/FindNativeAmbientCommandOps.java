// Private research index for unit command12 and ambient-pool references.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import java.io.PrintWriter;
public class FindNativeAmbientCommandOps extends GhidraScript{
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  InstructionIterator it=currentProgram.getListing().getInstructions(true);
  while(it.hasNext()){Instruction i=it.next();String s=i.toString();if((s.contains(" + 0x5c]")&&(s.endsWith(",0xc")||s.endsWith(",0xc0")))||s.contains("0x60b028")||s.contains("0x5860ce")){
   Function f=currentProgram.getFunctionManager().getFunctionContaining(i.getAddress());out.println(i.getAddress()+" "+s+" FUNCTION "+(f==null?"null":f.getEntryPoint()+" "+f.getName()));
  }}
 }}
}
