// Locate byte-field accesses for the private OBJECT contact delay audit.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import java.io.PrintWriter;
public class FindNativeObjectDelayOps extends GhidraScript{
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  InstructionIterator it=currentProgram.getListing().getInstructions(true);
  while(it.hasNext()){Instruction i=it.next();String text=i.toString();if(text.contains(" + 0x1f]")&&text.contains("byte ptr")){Function f=currentProgram.getFunctionManager().getFunctionContaining(i.getAddress());out.println(i.getAddress()+" "+text+" FUNCTION "+(f==null?"null":f.getEntryPoint()+" "+f.getName()));}}
 }}
}
