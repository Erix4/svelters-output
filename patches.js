import { handle_event } from './pkg/svelters_output.js';

function applyPatch(target, patchOp, nodeMap, fullMap) {
    if (patchOp.MountPage) {
        nodeMap[target] = document.createElement(patchOp.MountPage.tag);
        document.body.appendChild(nodeMap[target]);
    } else if (patchOp.MountTag) {
        nodeMap[target] = document.createElement(patchOp.MountTag.tag);
        nodeMap[patchOp.MountTag.parent_id].appendChild(nodeMap[target]);
    } else if (patchOp.MountEachItem) {
        const newNode = document.createElement(patchOp.MountEachItem.tag);
        nodeMap[target].push(newNode);
        nodeMap[patchOp.MountEachItem.parent_id].appendChild(newNode);
    } else if (patchOp.MountFragment) {
        const newNode = document.createElement(patchOp.MountFragment.tag);
        nodeMap[target] = [newNode];
        nodeMap[patchOp.MountFragment.parent_id].appendChild(newNode);
    } else if (patchOp == "MountEachFragment") {
        nodeMap[target] = [];
    } else if (patchOp.MountComment) {
        nodeMap[target] = document.createComment("");
        nodeMap[patchOp.MountComment.parent_id].appendChild(nodeMap[target]);
    } else if (patchOp.AddToMap) {
        nodeMap[target] = document.createElement(patchOp.AddToMap.tag);
    } else if (patchOp.InsertBefore) {
        nodeMap[patchOp.InsertBefore.parent_id].insertBefore(nodeMap[target], nodeMap[patchOp.InsertBefore.reference_id]);
    } else if (patchOp.MountEventListener) {
        nodeMap[target].addEventListener(patchOp.MountEventListener.event_type, (e) => {
            handle_js_event(e, fullMap, patchOp.MountEventListener.target_id_path);
        });
    } else if (patchOp.SetContent) {
        nodeMap[target].textContent = patchOp.SetContent.value;
    } else if (patchOp.SetAttribute) {
        nodeMap[target].setAttribute(patchOp.SetAttribute.name, patchOp.SetAttribute.value);
    } else if (patchOp.UnmountTag) {
        nodeMap[patchOp.UnmountTag.parent_id].removeChild(nodeMap[target]);
    } else if (patchOp.UnmountEachItem) {
        nodeMap[patchOp.UnmountEachItem.parent_id].removeChild(nodeMap[patchOp.UnmountEachItem.each_id][target]);
        nodeMap[patchOp.UnmountEachItem.each_id].splice(target, 1);
    }
}

export function applyPatchTree(patchTree, nodeMap, fullMap) {
    for (const node of patchTree) {
        if (node.Leaf) {
            const [target_id, operation] = node.Leaf;
            applyPatch(target_id, operation, nodeMap, fullMap);
        } else if (node.Node) {
            const [target_id, children] = node.Node;
            applyPatchTree(children, nodeMap[target_id], fullMap);
        }
    }
}

export async function handle_js_event(e, nodeMap, target_id) {
    const patches = handle_event(e, target_id);
    applyPatchTree(patches, nodeMap, nodeMap);
}