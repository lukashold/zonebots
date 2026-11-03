// Locate every direct reference into the two240-row20-byte command pools.
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.scalar.Scalar;
import ghidra.program.model.listing.*;
import java.io.*;
public class ZNativeOrderPoolReferences extends GhidraScript{
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){InstructionIterator it=currentProgram.getListing().getInstructions(true);while(it.hasNext()){Instruction i=it.next();boolean hit=false;for(int op=0;op<i.getNumOperands();op++){for(Object o:i.getOpObjects(op)){long v=o instanceof Scalar?((Scalar)o).getUnsignedValue():o instanceof Address?((Address)o).getOffset():-1;if(v>=0x5c6a40L&&v<0x5c8fc0L)hit=true;}}if(hit){Function f=currentProgram.getFunctionManager().getFunctionContaining(i.getAddress());out.println(i.getAddress()+" "+i+" FUNCTION "+(f==null?"NONE":f.getEntryPoint()));}}}}
}
